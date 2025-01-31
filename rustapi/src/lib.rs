use anyhow::Result;
use bindings::deps::component::compressor::compress::Compressor;
use spin_sdk::{
    http::{
        Headers, IncomingRequest, IncomingResponse, Method, OutgoingResponse, Request,
        ResponseOutparam,
    },
    http_component,
};

mod bindings;

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

    let mut incoming_response_body = res.take_body_stream();

    let outgoing_response = OutgoingResponse::new(Headers::from_list(
        &headers
            .into_iter()
            .filter(|(k, _)| k == "content-type")
            .collect::<Vec<_>>(),
    )?);

    let mut outgoing_response_body = outgoing_response.take_body();

    out.set(outgoing_response);

    // let compression_enabled = req.headers().get(&"Available-Dictionary".to_string());
    let compression_enabled = req.headers().has(&"Available-Dictionary".to_string());

    if compression_enabled {
        // compression logic
        let compressor = Compressor::new(10, "");

        let mut count = 0;
        while let Some(chunk) = incoming_response_body.next().await {
            count += 1;
            let chunk = chunk?;

            let buf = compressor.add_bytes(&chunk);
            // println!(
            //     "Chunk #{count}. Initial size {}; Compressed size: {}",
            //     &chunk.len(),
            //     buf.len()
            // );
            outgoing_response_body.send(buf).await?;
        }

        let f = compressor.finish();
        outgoing_response_body.send(f).await?;
    //
    } else {
        let mut count = 0;
        while let Some(chunk) = incoming_response_body.next().await {
            count += 1;
            let chunk = chunk?;
            // println!("Chunk #{count}. Initial size {}", &chunk.len(),);
            outgoing_response_body.send(chunk).await?;
        }
    }

    Ok(())
}
