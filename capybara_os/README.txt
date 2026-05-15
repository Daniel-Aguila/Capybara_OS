##Bootloader -> Kernel Handoff
The bootloader hands off the kernel arguments as memory type: LOADER_DATA.
This means that the Kernel must read the kernel arguments, copy them and then treat that memory space and reclaim it.
