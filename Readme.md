# A DLL Injector for Windows

This is small CLI utility that can inject a DLL into a foreign process.

## Installation
```shell
cargo install --git https://github.com/sidit77/inject.git
```

## Usage 
```
Usage: inject.exe [OPTIONS] <PATH> <PROCESS>

Arguments:
  <PATH>
          The path of the DLL file

  <PROCESS>
          The process name

Options:
  -p, --pid
          Interpret the process argument as PID

  -c, --copy
          Create a copy of the DLL before injecting to allow for easier overwriting

  -m, --mode <MODE>
          What mode to run the program in

          [default: inject]

          Possible values:
          - inject: Inject the DLL into the target process
          - eject:  Tries to eject the DLL from the target process
          - reload: Combination of `Eject` followed by `Inject`

  -l, --level <LEVEL>
          The log level

          [default: info]
          [possible values: off, error, warn, info, debug, trace]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## License
MIT