use anyhow::Result;
use spin_sdk::{
    http::{
        Headers, IncomingRequest, IncomingResponse, Method, OutgoingResponse, Request,
        ResponseOutparam,
    },
    http_component,
};

mod compressor {
    #![allow(missing_docs)]
    wit_bindgen::generate!({
        world: "deps",
        path: "../.wit/components/rustapi",
        with: {
            "wasi:io/error@0.2.0": spin_executor::bindings::wasi::io::error,
            "wasi:io/streams@0.2.0": spin_executor::bindings::wasi::io::streams,
            "wasi:io/poll@0.2.0": spin_executor::bindings::wasi::io::poll,
        }
    });
}
use crate::compressor::component::compressor::compress::Compressor;

use futures::{SinkExt, StreamExt};

#[http_component]
async fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
    stream(request, response_out)
        .await
        .expect("cannot stream file")
}

async fn stream(req: IncomingRequest, out: ResponseOutparam) -> Result<()> {
    let headers = req.headers().entries();
    let r = Request::builder()
        .method(Method::Get)
        .uri(format!("/static/assets/big.txt"))
        .build();

    // Send the request and await the response
    let res: IncomingResponse = spin_sdk::http::send(r).await?;

    let outgoing_response = OutgoingResponse::new(Headers::from_list(
        &headers
            .into_iter()
            .filter(|(k, _)| k == "content-type")
            .collect::<Vec<_>>(),
    )?);

    // let compression_enabled = req.headers().get(&"Available-Dictionary".to_string());
    let compression_enabled = req.headers().has(&"Available-Dictionary".to_string());

    if compression_enabled {
        let incoming_response_body = res.consume().unwrap();
        let input = incoming_response_body.stream().unwrap();
        let outgoing_response_body = outgoing_response.body().unwrap();
        let output = outgoing_response_body.write().unwrap();
        out.set(outgoing_response);

        // compression logic
        let compressor = Compressor::new(10, "");
        compressor.pipe_through(&input, &output);
    } else {
        let mut incoming_response_body = res.take_body_stream();
        let mut outgoing_response_body = outgoing_response.take_body();
        out.set(outgoing_response);
        let mut count = 0;
        while let Some(chunk) = incoming_response_body.next().await {
            count += 1;
            let chunk = chunk?;
            // println!("Chunk #{count}. Initial size {}", &chunk.len(),);
            outgoing_response_body.send(chunk).await?;
        }
    }

    println!("Done streaming");
    Ok(())
}
