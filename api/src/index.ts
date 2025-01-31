import { AutoRouter } from 'itty-router';
import Compressor from './compression';

let router = AutoRouter();


router
    // Route handler for clients requesting dictionaries.
    // Not using the default file server here because we need the additional header 'Use-As-Dictionary'.
    .get("/dictionaries/:dict", async (req) => {
        let { dict } = req.params;
        let res = await fetch(`/static/assets/${dict}`);
        let headers = new Headers(res.headers);

        // This currently sets this as a global dictionary for every asset.
        // This should be configurable per dictionary.
        headers.set("Use-As-Dictionary", `id=${dict}, match="/*", match-dest=("document" "frame")`);

        return new Response(res.body, { status: res.status, statusText: res.statusText, headers: headers });
    })

    .get("/stream/:file", async (req) => {
        // const res = await fetch("https://raw.githubusercontent.com/dscape/spell/refs/heads/master/test/resources/big.txt");
        let { file } = req.params;
        console.log(`file: ${file}`)
        const res = await fetch(`/static/assets/${file}`);

        const { status, body } = res;

        // If the request headers contained a dictionary ID, compress the outgoing stream.
        // Otherwise, stream the response unaltered.
        let id = req.headers.get("Dictionary-Id");
        if (req.headers.get("Available-Dictionary") && id) {
            console.log(`Requested compression with dictionary ${id}`);
            let compressor = new Compressor(10, id);
            let chunks = 0;
            let compressionStream = new TransformStream({
                transform(chunk, controller) {
                    let buf = compressor.addBytes(chunk);
                    console.log(`Chunk #${chunks}: initial chunk size ${chunk.length}; compressed size ${buf.length}`);
                    controller.enqueue(buf);
                    chunks++;
                }, flush(controller) {
                    let final = compressor.finish();
                    console.log(`Final size: ${final.length}`);
                    console.log(`Number of chunks: ${chunks}`);
                    controller.enqueue(final);
                }
            });

            let compressedBody = body!.pipeThrough(compressionStream);
            // let newHeaders = new Headers(res.headers);
            // newHeaders.delete("Content-Length");
            // newHeaders.set("Content-Encoding", "zstd");

            return new Response(compressedBody, { status });

        } else {

            let chunks = 0;
            // Create a TransformStream to log chunks and pass them along
            // This is not really needed to just stream the body, but helpful debugging why the above is failing.
            const transformStream = new TransformStream({
                transform(chunk, controller) {
                    chunks++;
                    console.log(`Chunk #${chunks}: initial chunk size ${chunk.length}`);
                    controller.enqueue(chunk); // Pass the chunk along
                }, flush(_controller) {
                    console.log(`Number of chunks: ${chunks}`);
                }
            });

            const transformedBody = body!.pipeThrough(transformStream);

            console.log("Returning the response body unmodified");
            return new Response(transformedBody, { status });
            // return new Response(body, { status, headers })
        }
    })

//@ts-ignore
addEventListener('fetch', async (event: FetchEvent) => {
    try {
        event.respondWith(router.fetch(event.request));
    } catch (e: any) {
        console.error(`Error: ${e}`);
    }
});

