# promise-stream-reader

[![Travis CI build status](https://travis-ci.org/princjef/promise-stream-reader.svg?branch=dev)](https://travis-ci.org/princjef/promise-stream-reader)
[![codecov](https://codecov.io/gh/princjef/promise-stream-reader/branch/dev/graph/badge.svg)](https://codecov.io/gh/princjef/promise-stream-reader)
[![npm version](https://img.shields.io/npm/v/promise-stream-reader.svg)](https://npmjs.org/package/promise-stream-reader)


Consume data from Node.js readable streams using promises and async/await to
control flow.

```
npm install promise-stream-reader
```

## Usage

The base export is a function that returns a writable stream to which you can
pipe your readable/duplex stream.

```js
const promiseStream = require('promise-stream-reader');

async function readStream(readableStream) {
    const reader = promiseStream();
    readableStream.pipe(reader);

    // Read 4 bytes of data from the stream
    const part1 = await reader.read(4);

    // Skip 10 bytes of data
    await reader.skip(10);

    // You can also have multiple read/skip calls pending at the same time. They
    // will be filled with data in the order they were initially requested.
    const [part2, part3] = await Promise.all([
        reader.read(6),
        reader.read(8)
    ]);

    // When you're all done, you can close up the streams
    readableStream.unpipe(reader);
    reader.destroy();
    readableStream.destroy();
}

readStream(/* some readable stream */)
    .catch(err => { /* don't forget to handle rejected promises */ });
```

*NOTE: If you're using Typescript, the base exported function is available under
the default export.*

## Why?

There are [two modes][stream two modes] for reading data from a readable stream
in Node.js, but each has limitations that can be frustrating for a final
consumer:

 * [`on('data')`][stream data event] (flowing) allows you to deal with data as
   it comes in, but you have to take care of packaging the data up into the
   chunks appropriate for your use case. You will also continue to receive data
   as fast as it comes in by default, even if you're not really ready for it.

 * [`read()`][stream read] (paused) allows you to explicitly request data but
   has no mechanism for asynchronous retrieval. As a result, it will return null
   if there is no data available yet, forcing you to poll until you get the data
   you want. You can combine this with listening to the `'readable'` event, but
   then you're using two patterns and still have to wrap it if you want to use
   promises.

This solution aims to strike a balance between the two, allowing you to request
the data you want when you need it and have the stream provide it to you as soon
as it's available. You get to process the data quickly without extra
manipulation or control flow required and can proceed at your own pace.

## Contributing

Want to contribute to the project? Go check out the [Contribution Guide](CONTRIBUTING.md) for instructions to set up your development environment, open
an issue and create a pull request.

[stream two modes]: https://nodejs.org/api/stream.html#stream_two_modes
[stream read]: https://nodejs.org/api/stream.html#stream_readable_read_size
[stream data event]: https://nodejs.org/api/stream.html#stream_event_data