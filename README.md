ABANDONED. Try [python-launcher](https://github.com/brettcannon/python-launcher) if you’re interested in a similar solution.

# Python Launcher for POSIX

[![Build Status](https://travis-ci.com/uranusjr/pylauncher-posix.svg?branch=master)](https://travis-ci.com/uranusjr/pylauncher-posix)

This projects implements a `py` executable that launches Python executables, akin to the `py.exe` launcher for Windows as first outlined by [PEP 397], and various PEPs subsequently.

[PEP 397]: https://www.python.org/dev/peps/pep-0397/


## Rationale

The idea is simple: Instead of running a Python command directly (and rely on the name to know what version it is), a `py` command is always used. The Python version you want is specified by an option, so that

* `py -2` runs Python 2.
* `py -3` runs Python 3.
* `py -3.5` runs Python 3.5.

and so on. This avoids relying on the `PATH` environment variable, which can be difficult to keep track of if you have a lot of different development environments on a machine.


## Finding a Python

Unlike Windows (from which this utility drew inspiration from), POSIX systems do not have a registry to declare what is installed. This tool, therefore, uses some heuristics to determine find Pythons.

Python installations are categorised into two kinds: managed, and executable.

### Managed Pythons

A managed Python is installed by a version manager, e.g. pyenv. These managers put all Python installations inside a directory (e.g. `$PYENV_ROOT/versions`). Each subdirectory inside is named after the version, and is the root prefix of that Python.

This tool looks for an environment variable, `PY_MANAGED_DIR`, to know where to look for managed Pythons. If you use pyenv, for example, you can set it like this:

```bash
export PY_MANAGED_DIR=$PYENV_ROOT/versions
```

So that installations in the directory can be found. You can also set multiple values seperated by `:` if you use multiple tools:

```bash
export PY_MANAGED_DIR=$PYENV_ROOT/versions:$ASDF_DATA_DIR/installs/python
```

The following managers are known to work at the current time:

* pyenv
* asdf
* Pythonz

Only stable CPython installations are supported.


### Pythons in PATH

The `PATH` environment variable is also inspected to find Pythons. This uses the customary `pythonX.Y` naming convention to tell what version an executable is. *The accuracy of the names is not checked.* `python3.5` will be identified as Python 3.5 (of unknown patch version), and `python3` as Python 3 (of unknown minor and patch version).


## Choosing a Python

A Python is chosen from the above findings with the following criteria:

1. The higher a version is, the better.
2. The more specified a version is, the better.
3. Managed over PATH.
4. The order of `PY_MANAGED_DIR` and `PATH` is respected.

Note that rule 2. means the result may not match the order in PATH. For `py -3`, for example, a `python3.4` will be preferred over `python3`, even if the latter is specified earlier. I personally think this is fine (and, to be honest, this is much easier to implement), but am open to changing it if someone makes a good argument.

Also, conforming to [PEP 486], if `py` is invoked (no version specifications) inside a virtual environment, the virtual environment’s Python is always used.

[PEP 486]: https://www.python.org/dev/peps/pep-0486/


## Installation

If you’re on macOS, a [Homebrew](https://brew.sh) [Tap](https://docs.brew.sh/Taps) is available:

    brew tap uranusjr/pythonup
    brew install pylauncher

To install from source, use [Cargo](https://crates.io/):

    cargo install pylauncher
