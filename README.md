
# rust-audio-analysis
RAA!

RAA is an audio analysis framework, for Rust. It aims to be relatively extensible. Audio analysis is done via chains, which have a source and multiple nodes.

The basic workflow is that you create a chain, add a source node to it and add analysis nodes to it. Then you can pick out values from the analysis nodes. This way, you can combine filters and analysis algorithms in any way you like. You can also use the same source node for multiple chains, thus saving on performance.

One might, for example, combine FFT with a high-pass filter and RMS with a peak detection and use the same source for both.

RAA is still work under progress and many of its features are missing.

Currently it has:

 - libsoundio support.
 - PortAudio support.
 - Basic structure for chains and nodes.
 - RMS node.
 - Server component for remote use.

Currently it's missing:

 - Reading audio from a file.
 - All interesting algorithms (FFT, high/low pass filters, etc)
 - Proper file structure & cleanup..

Examples coming at some point (sooner if there's interest for someone to contribute, later if there's not!)
