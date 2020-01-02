using System;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using System.Text;

namespace overwolf.plugins {
	public partial class MemoryReader {
		public static string StringFromNativeUtf8(IntPtr nativeUtf8) {
			var len = 0;
			while (Marshal.ReadByte(nativeUtf8, len) != 0) ++len;
			var buffer = new byte[len];
			Marshal.Copy(nativeUtf8, buffer, 0, buffer.Length);
			return Encoding.UTF8.GetString(buffer);
		}

		[StructLayout(LayoutKind.Sequential)]
		public struct ResultI32 {
			public Int32 value;
			public byte success;
		}

		[StructLayout(LayoutKind.Sequential)]
		public struct ResultCharPtr {
			public IntPtr value;
			public byte success;
		}

#if x86
		[DllImport("memory-reader.dll", EntryPoint="free_str")]
#elif x64
		[DllImport("memory-reader64.dll", EntryPoint="free_str")]
#else
#error undefined architecture
#endif
		static extern void free_str(IntPtr ptr);

#if x86
		[DllImport("memory-reader.dll", EntryPoint="process_path")]
#elif x64
		[DllImport("memory-reader64.dll", EntryPoint="process_path")]
#else
#error undefined architecture
#endif
		static extern ResultCharPtr process_path([MarshalAs(UnmanagedType.LPUTF8Str)] string processName);

#if x86
		[DllImport("memory-reader.dll", EntryPoint="read_i32")]
#elif x64
		[DllImport("memory-reader64.dll", EntryPoint="read_i32")]
#else
#error undefined architecture
#endif
		static extern ResultI32 read_i32(
			[MarshalAs(UnmanagedType.LPUTF8Str)] string processName,
			[MarshalAs(UnmanagedType.LPUTF8Str)] string moduleName,
			[MarshalAs(UnmanagedType.LPArray)] UIntPtr[] offsets,
			UInt64 offsetsLen
		);

		public void ProcessPath(string processName, Action<object> cb) {
			Task.Run(() => {
				var result = process_path(processName);

				if (result.success == 1) {
					cb(new { value = StringFromNativeUtf8(result.value), success = true });
					free_str(result.value);
				} else {
					cb(new { value = "", success = false });
				}
			});
		}

		public void ReadI32(string processName, string moduleName, int[] offsets, Action<object> cb) {
			Task.Run(() => {
				var result = read_i32(processName, moduleName, offsets.Select(i => (UIntPtr)i).ToArray(), (UInt64)offsets.Length);
				cb(new { value = result.value, success = (result.success == 1) });
			});
		}
	}
}
