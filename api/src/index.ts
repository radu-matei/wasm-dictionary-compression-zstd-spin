import { AutoRouter } from 'itty-router';
import { compress } from "@spin-deps/component-compressor"

let router = AutoRouter();

router
    // Route handler for clients requesting dictionaries.
    // Not using the default file server here because we need the additional header 'Use-As-Dictionary' if we expect
    // this to be used by browsers.
    .get("/dictionaries/:dict", async (req) => {
        let { dict } = req.params;
        let res = await fetch(`/static/assets/dictionaries/${dict}`);
        let headers = new Headers(res.headers);

        // This currently sets this as a global dictionary for every asset.
        // It should be configurable per dictionary.
        headers.set("Use-As-Dictionary", `id=${dict}, match="/*", match-dest=("document" "frame")`);
        return new Response(res.body, { status: res.status, statusText: res.statusText, headers: headers });
    })

    // Stream a file and if the right headers are present, compress it using the desired dictionary.
    .get("/stream/:file", async (req) => {
        let { file } = req.params;
        const res = await fetch(`/static/assets/train/${file}`);

        // If the request headers contained a dictionary ID, compress the outgoing stream.
        // Otherwise, stream the response unaltered.
        // This currently only supports Dictionary Compressed Zstandard streams.
        let acceptEncoding = req.headers.get("Accept-Encoding") || "";
        let id = req.headers.get("Dictionary-Id");
        if (req.headers.get("Available-Dictionary") && id && acceptEncoding.includes("dcz")) {
            console.log(`[compression]: Requested compression for file ${file} with dictionary ${id}`);

            // Instantiate the compressor and create a new transform stream.
            // Setting the default compression level here as 0.
            let compressor = new compress.Compressor(0, id);
            let compressionStream = new TransformStream({
                transform(chunk, controller) {
                    // for each chunk, compress thy bytes, then stream them
                    let buf = compressor.addBytes(chunk);
                    controller.enqueue(buf);
                }, flush(controller) {
                    let final = compressor.finish();
                    controller.enqueue(final);
                }
            });

            return new Response(res.body?.pipeThrough(compressionStream), { status: res.status });
            // The request did not have the right headers, so passing the stream along unaltered.
        } else {
            console.log(`[compression]: Streaming file ${file} unaltered`);
            return new Response(res.body, { status: res.status });
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
