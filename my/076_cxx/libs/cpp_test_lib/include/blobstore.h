#pragma once

#include <memory>
#include <unordered_map>
#include <set>
// Различные типы из CXX Rust библиотеки
#include <rust/cxx.h>

namespace blobstore{
    struct Blob{
        std::string data;
        std::set<std::string> tags;
    };

    struct Storage{
        std::unordered_map<uint64_t, Blob> blobs;
    };

    //////////////////////////////////////////////////

    // Здесь мы просто объявляем структурку из Rust кода
    struct MultiBuf;
    struct BlobMetadata;

    // Класс клиента
    class BlobstoreClient {
    public:
        // Важно! Все методы должны быть помечены как константные, иначе Rust не понимает их
        BlobstoreClient();
        uint64_t put(MultiBuf &buf) const;
        void tag(uint64_t blobId, rust::Str tag) const;
        BlobMetadata metadata(uint64_t blobId) const;

    private:
        // Отдельный shared_ptr для того, чтобы можно было вызывать из константных методов
        std::shared_ptr<Storage> _storage;
    };

    // Генератор UniquePtr клиента для управления временем жизни объекта в Rust
    std::unique_ptr<BlobstoreClient> new_blobstore_client();
}