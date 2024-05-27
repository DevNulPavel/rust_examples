Concurrent hash maps.

This crate implements concurrent hash maps, based on bucket-level multi-reader locks. It has
excellent performance characteristics¹ and supports resizing, in-place mutation and more.

The API derives directly from `std::collections::HashMap`, giving it a familiar feel.

¹Note that it heavily depends on the behavior of your program, but in most cases, it's really
 good. In some (rare) cases you might want atomic hash maps instead.

# How it works

`chashmap` is not lockless, but it distributes locks across the map such that lock contentions
(which is what could make accesses expensive) are very rare.

Hash maps consists of so called "buckets", which each defines a potential entry in the table.
The bucket of some key-value pair is determined by the hash of the key. By holding a read-write
lock for each bucket, we ensure that you will generally be able to insert, read, modify, etc.
with only one or two locking subroutines.

There is a special-case: reallocation. When the table is filled up such that very few buckets
are free (note that this is "very few" and not "no", since the load factor shouldn't get too
high as it hurts performance), a global lock is obtained while rehashing the table. This is
pretty inefficient, but it rarely happens, and due to the adaptive nature of the capacity, it
will only happen a few times when the map has just been initialized.

## Collision resolution

When two hashes collide, they cannot share the same bucket, so there must be an algorithm which
can resolve collisions. In our case, we use linear probing, which means that we take the bucket
following it, and repeat until we find a free bucket.

This method is far from ideal, but superior methods like Robin-Hood hashing works poorly (if at
all) in a concurrent structure.

# The API

The API should feel very familiar, if you are used to the libstd hash map implementation. They
share many of the methods, and I've carefully made sure that all the items, which have similarly
named items in libstd, matches in semantics and behavior.
