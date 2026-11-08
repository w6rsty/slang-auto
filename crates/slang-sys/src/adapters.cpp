#include <atomic>
#include <cstring>
#include <filesystem>
#include <slang.h>
#include <string>
#include <thread>
using Destroy = void (*)(void *);
using Load = SlangResult (*)(void *, const char *, ISlangBlob **);
static bool same(const SlangUUID &a, const SlangUUID &b) {
  return std::memcmp(&a, &b, sizeof(a)) == 0;
}
class RustBlob final : public ISlangBlob {
  std::atomic<uint32_t> refs{1};
  const void *data;
  size_t size;
  void *context;
  Destroy destroy;

public:
  RustBlob(const void *d, size_t s, void *c, Destroy f)
      : data(d), size(s), context(c), destroy(f) {}
  ~RustBlob() { destroy(context); }
  SlangResult SLANG_MCALL queryInterface(const SlangUUID &id,
                                         void **out) noexcept override {
    *out = nullptr;
    if (same(id, ISlangUnknown::getTypeGuid()) ||
        same(id, ISlangBlob::getTypeGuid())) {
      *out = static_cast<ISlangBlob *>(this);
      addRef();
      return SLANG_OK;
    }
    return SLANG_E_NO_INTERFACE;
  }
  uint32_t SLANG_MCALL addRef() noexcept override { return ++refs; }
  uint32_t SLANG_MCALL release() noexcept override {
    auto n = --refs;
    if (!n)
      delete this;
    return n;
  }
  const void *SLANG_MCALL getBufferPointer() noexcept override { return data; }
  size_t SLANG_MCALL getBufferSize() noexcept override { return size; }
};
static SlangResult pathBlob(const std::filesystem::path &path,
                            ISlangBlob **out) {
  auto text = new std::string(path.lexically_normal().u8string());
  *out = new RustBlob(text->c_str(), text->size() + 1, text,
                      [](void *p) { delete static_cast<std::string *>(p); });
  return SLANG_OK;
}
class RustFileSystem final : public ISlangFileSystemExt {
  std::atomic<uint32_t> refs{1};
  std::thread::id owner = std::this_thread::get_id();
  void *context;
  Load load;
  Destroy destroy;

public:
  RustFileSystem(void *c, Load l, Destroy d)
      : context(c), load(l), destroy(d) {}
  ~RustFileSystem() {
    if (owner != std::this_thread::get_id())
      std::terminate();
    destroy(context);
  }
  void *SLANG_MCALL castAs(const SlangUUID &id) noexcept override {
    if (same(id, ISlangUnknown::getTypeGuid()) ||
        same(id, ISlangCastable::getTypeGuid()) ||
        same(id, ISlangFileSystem::getTypeGuid()))
      return static_cast<ISlangFileSystem *>(this);
    if (same(id, ISlangFileSystemExt::getTypeGuid()))
      return static_cast<ISlangFileSystemExt *>(this);
    return nullptr;
  }
  SlangResult SLANG_MCALL queryInterface(const SlangUUID &id,
                                         void **out) noexcept override {
    *out = castAs(id);
    if (!*out)
      return SLANG_E_NO_INTERFACE;
    addRef();
    return SLANG_OK;
  }
  uint32_t SLANG_MCALL addRef() noexcept override { return ++refs; }
  uint32_t SLANG_MCALL release() noexcept override {
    auto n = --refs;
    if (!n)
      delete this;
    return n;
  }
  SlangResult SLANG_MCALL loadFile(const char *path,
                                   ISlangBlob **out) noexcept override {
    *out = nullptr;
    if (owner != std::this_thread::get_id())
      return SLANG_E_NOT_AVAILABLE;
    auto status = load(context, path, out);
    return status == 1 ? SLANG_E_NOT_FOUND : status;
  }
  SlangResult SLANG_MCALL
  getFileUniqueIdentity(const char *path, ISlangBlob **out) noexcept override {
    *out = nullptr;
    ISlangBlob *blob = nullptr;
    auto status = loadFile(path, &blob);
    if (status < 0)
      return status;
    blob->release();
    try {
      return pathBlob(std::filesystem::absolute(std::filesystem::u8path(path)),
                      out);
    } catch (...) {
      return SLANG_FAIL;
    }
  }
  SlangResult SLANG_MCALL calcCombinedPath(SlangPathType type, const char *from,
                                           const char *path,
                                           ISlangBlob **out) noexcept override {
    *out = nullptr;
    try {
      auto base = std::filesystem::u8path(from);
      if (type == SLANG_PATH_TYPE_FILE)
        base = base.parent_path();
      return pathBlob(base / std::filesystem::u8path(path), out);
    } catch (...) {
      return SLANG_FAIL;
    }
  }
  SlangResult SLANG_MCALL getPathType(const char *path,
                                      SlangPathType *out) noexcept override {
    ISlangBlob *blob = nullptr;
    auto status = loadFile(path, &blob);
    if (status >= 0) {
      blob->release();
      *out = SLANG_PATH_TYPE_FILE;
    }
    return status;
  }
  SlangResult SLANG_MCALL getPath(PathKind, const char *path,
                                  ISlangBlob **out) noexcept override {
    *out = nullptr;
    try {
      return pathBlob(std::filesystem::absolute(std::filesystem::u8path(path)),
                      out);
    } catch (...) {
      return SLANG_FAIL;
    }
  }
  void SLANG_MCALL clearCache() noexcept override {}
  SlangResult SLANG_MCALL enumeratePathContents(const char *,
                                                FileSystemContentsCallBack,
                                                void *) noexcept override {
    return SLANG_E_NOT_IMPLEMENTED;
  }
  OSPathKind SLANG_MCALL getOSPathKind() noexcept override {
    return OSPathKind::None;
  }
};
extern "C" ISlangBlob *slang_auto_blob(const void *data, size_t size,
                                       void *context,
                                       Destroy destroy) noexcept {
  return new RustBlob(data, size, context, destroy);
}
extern "C" ISlangFileSystem *slang_auto_filesystem(void *context, Load load,
                                                   Destroy destroy) noexcept {
  return new RustFileSystem(context, load, destroy);
}
