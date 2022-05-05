# vcd2v

Generate Verilog stimulus from [VCD](https://en.wikipedia.org/wiki/Value_change_dump) input.

## Usage

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
