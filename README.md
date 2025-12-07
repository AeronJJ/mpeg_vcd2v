# Credit
Original source code credited to Andrew Kay [https://git.sr.ht/~ajk/vcd2v](https://git.sr.ht/~ajk/vcd2v)

This program has been modified to flow better in my mpeg-recorder project.

Changes from source:
- Reads VCD file in program, rather than reading from terminal buffer
- Read signal map file
- Scans VCD for all signals and asks user to include them, only if no signal map or selection is provided

# vcd2v

Generate Verilog stimulus from [VCD](https://en.wikipedia.org/wiki/Value_change_dump) input.

## Build

```
git clone https://github.com/AeronJJ/mpeg_vcd2v.git
cargo build
```
Executable found: ./target/debug/vcd2v

## Usage

```bash
vcd2v [OPTIONS] -i <input_file> <selection> > <outputfile>
```

### Args
```
-i, --input <input_file>           Input .vcd file
-t, --time [START][:END]           Start and end times
-s, --scale <value>                Multiply time by scaling factor
-m, --signal_map <signal_map_file> Signal Map file, see below
-h, --help                         Print help information
```

### Signal Map File
.sm file type, raw text

Each line specifies a new signal in the VCD with an optional alias prefix:
```text
# Aliased:
mpeg_sync_i=libsigrok.Sync
mpeg_valid_i=libsigrok.Valid
mpeg_data_8_i[0]=libsigrok.D0
# Raw:
libsigrok.D1
libsigrok.D2
```

### Examples
```bash
# Basic usage
vcd2v -i input.vcd > output.vcd

# Time range
vcd2v --time 0:500 -i input.vcd > output.vcd

# Scale values
vcd2v --scale 0.1 -i input.vcd > scaled.sv

# Combine map + manual selections
vcd2v --signal_map map.sm libsigrok.clk libsigrok.reset -i input.vcd > output.vcd
```



So, you have a VCD file containing signals (`libsigrok.D1` and `libsigrok.D3`), you want to convert these to a series of Verilog delay and assignment statements:

```
cat capture.vcd | vcd2v libsigrok.D1 libsigrok.D3
```

This will result in something like:

```
D1 = 0;
D3 = 1;
#375;
D1 = 1;
#500;
D1 = 0;
#125;
D1 = 1;
D3 = 0;
...
```

You can rename the Verilog variables used for the signals:

```
cat capture.vcd | vcd2v a=libsigrok.D1 b=libsigrok.D3
```

This will result in something like:

```
a = 0;
b = 1;
#375;
a = 1;
#500;
a = 0;
#125;
a = 1;
b = 0;
...
```

You can export only a portion of the VCD file using the `--time` argument.

You can apply a multiplier to the delays using the `--scale` argument.
