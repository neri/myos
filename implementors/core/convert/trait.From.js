(function() {var implementors = {};
implementors["kernel"] = [{"text":"impl From&lt;usize&gt; for <a class=\"enum\" href=\"kernel/arch/cpu/enum.PrivilegeLevel.html\" title=\"enum kernel::arch::cpu::PrivilegeLevel\">PrivilegeLevel</a>","synthetic":false,"types":["kernel::arch::x86_64::cpu::PrivilegeLevel"]},{"text":"impl From&lt;<a class=\"enum\" href=\"kernel/arch/cpu/enum.ExceptionType.html\" title=\"enum kernel::arch::cpu::ExceptionType\">ExceptionType</a>&gt; for <a class=\"struct\" href=\"kernel/arch/cpu/struct.InterruptVector.html\" title=\"struct kernel::arch::cpu::InterruptVector\">InterruptVector</a>","synthetic":false,"types":["kernel::arch::x86_64::cpu::InterruptVector"]},{"text":"impl From&lt;u8&gt; for <a class=\"struct\" href=\"kernel/bus/pci/struct.PciCapabilityId.html\" title=\"struct kernel::bus::pci::PciCapabilityId\">PciCapabilityId</a>","synthetic":false,"types":["kernel::bus::pci::pci::PciCapabilityId"]},{"text":"impl From&lt;usize&gt; for <a class=\"enum\" href=\"kernel/fs/enum.Whence.html\" title=\"enum kernel::fs::Whence\">Whence</a>","synthetic":false,"types":["kernel::fs::filesys::Whence"]},{"text":"impl&lt;T&gt; From&lt;T&gt; for <a class=\"struct\" href=\"kernel/sync/struct.Mutex.html\" title=\"struct kernel::sync::Mutex\">Mutex</a>&lt;T&gt;","synthetic":false,"types":["kernel::sync::mutex::Mutex"]},{"text":"impl&lt;T&gt; From&lt;<a class=\"struct\" href=\"kernel/sync/struct.PoisonError.html\" title=\"struct kernel::sync::PoisonError\">PoisonError</a>&lt;T&gt;&gt; for <a class=\"enum\" href=\"kernel/sync/enum.TryLockError.html\" title=\"enum kernel::sync::TryLockError\">TryLockError</a>&lt;T&gt;","synthetic":false,"types":["kernel::sync::TryLockError"]},{"text":"impl From&lt;Duration&gt; for <a class=\"struct\" href=\"kernel/task/scheduler/struct.TimeSpec.html\" title=\"struct kernel::task::scheduler::TimeSpec\">TimeSpec</a>","synthetic":false,"types":["kernel::task::scheduler::TimeSpec"]}];
implementors["megstd"] = [{"text":"impl&lt;'a&gt; From&lt;&amp;'a <a class=\"struct\" href=\"megstd/drawing/struct.Bitmap32.html\" title=\"struct megstd::drawing::Bitmap32\">Bitmap32</a>&lt;'a&gt;&gt; for <a class=\"struct\" href=\"megstd/drawing/struct.ConstBitmap32.html\" title=\"struct megstd::drawing::ConstBitmap32\">ConstBitmap32</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::ConstBitmap32"]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a <a class=\"struct\" href=\"megstd/drawing/struct.ConstBitmap8.html\" title=\"struct megstd::drawing::ConstBitmap8\">ConstBitmap8</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.ConstBitmap.html\" title=\"enum megstd::drawing::ConstBitmap\">ConstBitmap</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::ConstBitmap"]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a <a class=\"struct\" href=\"megstd/drawing/struct.Bitmap8.html\" title=\"struct megstd::drawing::Bitmap8\">Bitmap8</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.ConstBitmap.html\" title=\"enum megstd::drawing::ConstBitmap\">ConstBitmap</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::ConstBitmap"]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a <a class=\"struct\" href=\"megstd/drawing/struct.ConstBitmap32.html\" title=\"struct megstd::drawing::ConstBitmap32\">ConstBitmap32</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.ConstBitmap.html\" title=\"enum megstd::drawing::ConstBitmap\">ConstBitmap</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::ConstBitmap"]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a <a class=\"struct\" href=\"megstd/drawing/struct.Bitmap32.html\" title=\"struct megstd::drawing::Bitmap32\">Bitmap32</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.ConstBitmap.html\" title=\"enum megstd::drawing::ConstBitmap\">ConstBitmap</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::ConstBitmap"]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a mut <a class=\"struct\" href=\"megstd/drawing/struct.Bitmap8.html\" title=\"struct megstd::drawing::Bitmap8\">Bitmap8</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.Bitmap.html\" title=\"enum megstd::drawing::Bitmap\">Bitmap</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::Bitmap"]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a mut <a class=\"struct\" href=\"megstd/drawing/struct.Bitmap32.html\" title=\"struct megstd::drawing::Bitmap32\">Bitmap32</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.Bitmap.html\" title=\"enum megstd::drawing::Bitmap\">Bitmap</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::Bitmap"]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a mut <a class=\"enum\" href=\"megstd/drawing/enum.OwnedBitmap.html\" title=\"enum megstd::drawing::OwnedBitmap\">OwnedBitmap</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.Bitmap.html\" title=\"enum megstd::drawing::Bitmap\">Bitmap</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::Bitmap"]},{"text":"impl&lt;'a&gt; From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.Bitmap8.html\" title=\"struct megstd::drawing::Bitmap8\">Bitmap8</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.OwnedBitmap.html\" title=\"enum megstd::drawing::OwnedBitmap\">OwnedBitmap</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::OwnedBitmap"]},{"text":"impl&lt;'a&gt; From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.Bitmap32.html\" title=\"struct megstd::drawing::Bitmap32\">Bitmap32</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.OwnedBitmap.html\" title=\"enum megstd::drawing::OwnedBitmap\">OwnedBitmap</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::OwnedBitmap"]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a mut <a class=\"enum\" href=\"megstd/drawing/enum.BoxedBitmap.html\" title=\"enum megstd::drawing::BoxedBitmap\">BoxedBitmap</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.Bitmap.html\" title=\"enum megstd::drawing::Bitmap\">Bitmap</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::Bitmap"]},{"text":"impl&lt;'a&gt; From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.BoxedBitmap8.html\" title=\"struct megstd::drawing::BoxedBitmap8\">BoxedBitmap8</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.BoxedBitmap.html\" title=\"enum megstd::drawing::BoxedBitmap\">BoxedBitmap</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::BoxedBitmap"]},{"text":"impl&lt;'a&gt; From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.BoxedBitmap32.html\" title=\"struct megstd::drawing::BoxedBitmap32\">BoxedBitmap32</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.BoxedBitmap.html\" title=\"enum megstd::drawing::BoxedBitmap\">BoxedBitmap</a>&lt;'a&gt;","synthetic":false,"types":["megstd::drawing::bitmap::BoxedBitmap"]},{"text":"impl From&lt;u8&gt; for <a class=\"struct\" href=\"megstd/drawing/struct.IndexedColor.html\" title=\"struct megstd::drawing::IndexedColor\">IndexedColor</a>","synthetic":false,"types":["megstd::drawing::color::IndexedColor"]},{"text":"impl From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.IndexedColor.html\" title=\"struct megstd::drawing::IndexedColor\">IndexedColor</a>&gt; for <a class=\"struct\" href=\"megstd/drawing/struct.TrueColor.html\" title=\"struct megstd::drawing::TrueColor\">TrueColor</a>","synthetic":false,"types":["megstd::drawing::color::TrueColor"]},{"text":"impl From&lt;u32&gt; for <a class=\"struct\" href=\"megstd/drawing/struct.TrueColor.html\" title=\"struct megstd::drawing::TrueColor\">TrueColor</a>","synthetic":false,"types":["megstd::drawing::color::TrueColor"]},{"text":"impl From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.TrueColor.html\" title=\"struct megstd::drawing::TrueColor\">TrueColor</a>&gt; for <a class=\"struct\" href=\"megstd/drawing/struct.IndexedColor.html\" title=\"struct megstd::drawing::IndexedColor\">IndexedColor</a>","synthetic":false,"types":["megstd::drawing::color::IndexedColor"]},{"text":"impl From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.TrueColor.html\" title=\"struct megstd::drawing::TrueColor\">TrueColor</a>&gt; for <a class=\"struct\" href=\"megstd/drawing/struct.ColorComponents.html\" title=\"struct megstd::drawing::ColorComponents\">ColorComponents</a>","synthetic":false,"types":["megstd::drawing::color::ColorComponents"]},{"text":"impl From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.ColorComponents.html\" title=\"struct megstd::drawing::ColorComponents\">ColorComponents</a>&gt; for <a class=\"struct\" href=\"megstd/drawing/struct.TrueColor.html\" title=\"struct megstd::drawing::TrueColor\">TrueColor</a>","synthetic":false,"types":["megstd::drawing::color::TrueColor"]},{"text":"impl From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.TrueColor.html\" title=\"struct megstd::drawing::TrueColor\">TrueColor</a>&gt; for <a class=\"struct\" href=\"megstd/drawing/struct.DeepColor30.html\" title=\"struct megstd::drawing::DeepColor30\">DeepColor30</a>","synthetic":false,"types":["megstd::drawing::color::DeepColor30"]},{"text":"impl From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.DeepColor30.html\" title=\"struct megstd::drawing::DeepColor30\">DeepColor30</a>&gt; for <a class=\"struct\" href=\"megstd/drawing/struct.TrueColor.html\" title=\"struct megstd::drawing::TrueColor\">TrueColor</a>","synthetic":false,"types":["megstd::drawing::color::TrueColor"]},{"text":"impl From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.IndexedColor.html\" title=\"struct megstd::drawing::IndexedColor\">IndexedColor</a>&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.SomeColor.html\" title=\"enum megstd::drawing::SomeColor\">SomeColor</a>","synthetic":false,"types":["megstd::drawing::color::SomeColor"]},{"text":"impl From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.TrueColor.html\" title=\"struct megstd::drawing::TrueColor\">TrueColor</a>&gt; for <a class=\"enum\" href=\"megstd/drawing/enum.SomeColor.html\" title=\"enum megstd::drawing::SomeColor\">SomeColor</a>","synthetic":false,"types":["megstd::drawing::color::SomeColor"]},{"text":"impl From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.Size.html\" title=\"struct megstd::drawing::Size\">Size</a>&gt; for <a class=\"struct\" href=\"megstd/drawing/struct.Rect.html\" title=\"struct megstd::drawing::Rect\">Rect</a>","synthetic":false,"types":["megstd::drawing::coords::Rect"]},{"text":"impl From&lt;<a class=\"struct\" href=\"megstd/drawing/struct.Coordinates.html\" title=\"struct megstd::drawing::Coordinates\">Coordinates</a>&gt; for <a class=\"struct\" href=\"megstd/drawing/struct.Rect.html\" title=\"struct megstd::drawing::Rect\">Rect</a>","synthetic":false,"types":["megstd::drawing::coords::Rect"]},{"text":"impl From&lt;<a class=\"enum\" href=\"megstd/io/enum.ErrorKind.html\" title=\"enum megstd::io::ErrorKind\">ErrorKind</a>&gt; for <a class=\"struct\" href=\"megstd/io/struct.Error.html\" title=\"struct megstd::io::Error\">Error</a>","synthetic":false,"types":["megstd::io::error::Error"]},{"text":"impl From&lt;<a class=\"struct\" href=\"megstd/struct.OsString.html\" title=\"struct megstd::OsString\">OsString</a>&gt; for <a class=\"struct\" href=\"megstd/path/struct.PathBuf.html\" title=\"struct megstd::path::PathBuf\">PathBuf</a>","synthetic":false,"types":["megstd::path::PathBuf"]},{"text":"impl From&lt;<a class=\"struct\" href=\"megstd/path/struct.PathBuf.html\" title=\"struct megstd::path::PathBuf\">PathBuf</a>&gt; for <a class=\"struct\" href=\"megstd/struct.OsString.html\" title=\"struct megstd::OsString\">OsString</a>","synthetic":false,"types":["megstd::osstr::OsString"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()