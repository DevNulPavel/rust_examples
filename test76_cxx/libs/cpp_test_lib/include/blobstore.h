#pragma once

#include <memory>

namespace blobstore{
    // Здесь мы просто объявляем структурку из Rust кода
    struct MultiBuf;

    // Класс клиента
    class BlobstoreClient {
    public:
        BlobstoreClient();
        uint64_t put(MultiBuf &buf) const;
    };

    // Генератор UniquePtr клиента для управления временем жизни объекта в Rust
    std::unique_ptr<BlobstoreClient> new_blobstore_client();
}