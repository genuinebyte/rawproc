# rawproc

I wanted to try and process the raw NEF files that my camera gave me using my own code, so I wrote
some. Then I put it into a library and now you're reading the readme for that library.

The interpolation algorithm in use currently assumes an RGGB color filter array. This'll probably
work for your camera, but it also might not.

The docs are kind of lacking (but will be improved!), so if you've somehow found this repository
and want to use this crate, the code over in the [easyraw repository][easyraw-github] is a pretty
good example.

[easyraw-github]: https://github.com/gennyble/easyraw
