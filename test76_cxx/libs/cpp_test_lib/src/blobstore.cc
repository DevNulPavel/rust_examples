// Важно! Путь указывается полный, включая имя нашего проекта в виде корня
#include <test76_cxx/libs/cpp_test_lib/include/blobstore.h>
// Путь к сгенерированному описанию структурок из Rust кода
#include <test76_cxx/src/main.rs.h>
// Из стандарной библиотеки C++
#include <functional>
#include <string>

namespace blobstore{

    //////////////////////////////////////////////////////////////////////////////////////////

    BlobstoreClient::BlobstoreClient():
        _storage(std::make_shared<Storage>()) {
    }

    // Выгружаем данные с помощью переданного из Rust итератора
    uint64_t BlobstoreClient::put(MultiBuf &buf) const {
        // Обходим переданный итератор до конца
        std::string contents;
        while (true) {
            // Вызываем Rust функцию с буффером
            auto chunk = buf.next_chunk();
            if (chunk.size() == 0) {
                break;
            }
            // Кладем данные из Rust в локальный С++ буффер
            contents.append(reinterpret_cast<const char *>(chunk.data()), chunk.size());
        }

        // Симулируем какую-то работу, вычисляя 64-битный хеш от переданнных данных
        auto blobid = std::hash<std::string>{}(contents);

        // Возвращаем наш хеш в Rust
        return blobid;
    }

    // Add tag to an existing blob.
    void BlobstoreClient::tag(uint64_t blobId, rust::Str tag) const {
        _storage->blobs[blobId].tags.emplace(tag);
    }

    // Retrieve metadata about a blob.
    BlobMetadata BlobstoreClient::metadata(uint64_t blobId) const {
        // Пустые метаданные
        BlobMetadata metadata{};

        // Ищем нужный blob
        auto blobIt = _storage->blobs.find(blobId);
        if (blobIt != _storage->blobs.end()) {
            metadata.size = blobIt->second.data.size();
            std::for_each(blobIt->second.tags.cbegin(), blobIt->second.tags.cend(),
                        [&](auto &t) { metadata.tags.emplace_back(t); });
        }

        return metadata;
    }

    std::unique_ptr<BlobstoreClient> new_blobstore_client() {
        return std::unique_ptr<BlobstoreClient>(new BlobstoreClient());
    }
}