# Dots-and-boxes

This repository is dedicated to [the dots-and-boxes
game](https://en.wikipedia.org/wiki/Dots_and_Boxes).

The main thing here is a draft of a paper I am writing on the game. A
PDF version of this paper is available
[here](https://drive.google.com/file/d/17FfeXFUwnBBKf1XHLdn83dD_sf7kzaMK/view?usp=sharing),
though this may not always be up-to-date with the head of this
repository (the commit used to generate the PDF may be found in the
footnote at the end).

There is also an engine written in the [Rust programming
language](https://www.rust-lang.org/), which I intend to assist with
calculations related to this game. However this engine is currently in
very early stages of development.

I would be very grateful to learn of any errors in any of this
material.

# Building the paper

## On Linux

To build the PDF from the source in this repository, you need a full
installation of [LaTeX](http://latex-project.org/), as well as Python
3 and Inkscape to generate some of the diagrams, and `make` to
coordinate the build. E.g. on Fedora, `dnf install texlive texlive-svg
inkscape pstoedit make`, `cd` into this repository, and run
`make`.

To perform [Pylint](https://www.pylint.org/) checks on the Python
scripts, run `make pylint` (requires `python3-pylint`). It
produces quite a bit of output so you may want to pipe it through
`less`.

## Using Docker

Docker helps achieve a reproducible build procedure which works
uniformly from any platform with Docker client support (e.g. Linux,
Mac or Windows) and which doesn't require you to install any packages
(besides Docker itself) on your host system.

First you will need [Docker](https://docker.com/) installed and
working: I would highly recommend you work through the tutorial for
your platform and run some examples to build some basic familiarity
with Docker on your computer before attempting this.

The basic procedure is:

* Build a base image containing the required external packages (which
is slow, as it requires hundreds of packages to be downloaded, but you
only have to do it once)
* Perform the build in a container starting from that base image
* Extract any files you may want from that container.

To do this, run the following commands from the root of this repo.
(There may be some slight differences between platforms, e.g. on Linux
you may need to ```sudo``` in order to access the Docker daemon on the
local host - hence the above advice to get some basic familiarity with
Docker on your system first.)

* ```docker build -t dabbase -f docker/Dockerfile.base docker```
* ```docker build -t dabbuild -f docker/Dockerfile.build .```
* ```docker run --name dab dabbuild make```
* ```docker cp dab:/dotsandboxes/dotsandboxes.pdf .```

If you want to examine the results more closely, you can commit the
```dab``` container to a new image and run a shell in it:

* ```docker commit dab dabresult```
* ```docker run --rm -ti dabresult /bin/bash```

And when you're finished:

```docker rm dab```

If you change the source files on your local system, you will need to
delete the ```dab``` container, and regenerate the ```dabbuild```
image again (i.e. repeat from the second step). You can avoid this by
instead running a throw-away container in privileged mode and mounting
your local folder into it as a volume:

```docker run --rm -ti --privileged -v `pwd`:/dotsandboxes -w /dotsandboxes dabbase make```

However this may not work on all platforms.

# Building the engine

For now, the engine is built separately from the paper. It uses the
standard Cargo mechanism for the Rust programming language.

First you will need to install the Rust toolchain, if you do not
already have it, according to the instructions on [the Rust
website](https://www.rust-lang.org/).

Then, from the ```engine``` directory, run ```cargo run 3 2``` to
start a new 3-by-2 game. Type ```help``` to see a list of available
commands.

You can also put a list of commands in a file, with the dimensions on
the first line. For example ```cargo run p50bl.pos``` to start from
the 3-by-2 corner discussed in the paper.

Other actions are standard to Cargo. For example ```cargo test``` runs
the unit tests, and ```cargo bench``` runs the benchmarks.

# Licensing

All material in this repository is Copyright 2016-2019 Andrew Medworth
(github@medworth.org.uk).

The Dots-and-Boxes paper is licensed under a Creative Commons license
as follows.

<a rel="license" href="http://creativecommons.org/licenses/by-nc-sa/4.0/"><img alt="Creative Commons License" style="border-width:0" src="https://i.creativecommons.org/l/by-nc-sa/4.0/88x31.png" /></a><br /><span xmlns:dct="http://purl.org/dc/terms/" property="dct:title">Dots-and-Boxes</span> by <a xmlns:cc="http://creativecommons.org/ns#" href="https://github.com/amdw/dotsandboxes" property="cc:attributionName" rel="cc:attributionURL">Andrew Medworth</a> is licensed under a <a rel="license" href="http://creativecommons.org/licenses/by-nc-sa/4.0/">Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License</a>.<br />Based on a work at <a xmlns:dct="http://purl.org/dc/terms/" href="https://github.com/amdw/dotsandboxes" rel="dct:source">https://github.com/amdw/dotsandboxes</a>.

The software in this repository is licensed under the GNU Affero
General Public License, as per the file LICENSE.txt.

I am happy to consider requests to re-license this software under a
non-copyleft open source license. Just get in touch with me to let me
know what you want to do and how the current license is
inconveniencing you.
