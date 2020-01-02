# memory-reader

A little library for simple external memory reading from Overwolf apps.

This has been in use internally for quite a while, but the API could definitely be more broad (strings, u32/64, etc) and it will be expanded if there is interest. Feature requests and bug/issue reports welcome.

## Usage:
Assuming the convenient plugin, something like:
```javascript
const processPathRes = MemoryReader.ProcessPath("csgo.exe");
if (processPathRes.success === true) {
	// ...
}

// ...

const healthRes = MemoryReader.ReadI32("csgo.exe", "client_panorama.dll", [hazeOffsets.dwClientState, 0x40, 0x108]);
if (healthRes.success === true) {
	// ...
}
```
