from basemap import BaseHashMap

class ChainingHashMap(BaseHashMap):
    def __init__(self, capacity=8):
        self.capacity = capacity
        self.buckets = [[] for _ in range(capacity)]
        self.size = 0
        self.last_probe_count = 0

    def _hash(self, key):
        ## Use python's built-in hash and then modulo self.capacity
        return hash(key) % self.capacity

    def set(self, key, value):
        hash_idx = self._hash(key=key)
        ## Reference to list not copy
        bucket = self.buckets[hash_idx]
        for (i, (k, v)) in enumerate(bucket):
            if k == key:
                bucket[i] = (key, value)
                return
        bucket.append((key, value))
        self.size += 1

    def get(self, key):
        self.last_probe_count = 0
        hash_idx = self._hash(key=key)
        ## Reference to list not copy
        bucket = self.buckets[hash_idx]
        for (i, (k, v)) in enumerate(bucket):
            self.last_probe_count += 1
            if k == key:
                return v
        return None

    def has(self, key):
        hash_idx = self._hash(key=key)
        ## Reference to list not copy
        bucket = self.buckets[hash_idx]
        for (i, (k, v)) in enumerate(bucket):
            if k == key:
                return True
        return False

    def delete(self, key):
        hash_idx = self._hash(key=key)
        ## Reference to list not copy
        bucket = self.buckets[hash_idx]
        to_delete_idx = -1
        for (i, (k, v)) in enumerate(bucket):
            if k == key:
                to_delete_idx = i
                break

        if to_delete_idx >= 0:
            bucket.pop(to_delete_idx)
            self.size -= 1

if __name__ == "__main__":
    hm = ChainingHashMap(8)

    # Test set and get
    hm.set("name", "Alice")
    hm.set("age", 25)
    assert hm.get("name") == "Alice", "get should return Alice"
    assert hm.get("age") == 25, "get should return 25"

    # Test update
    hm.set("name", "Bob")
    assert hm.get("name") == "Bob", "update should change value"

    # Test has
    assert hm.has("name") == True, "has should return True for existing key"
    assert hm.has("city") == False, "has should return False for missing key"

    # Test delete
    hm.delete("age")
    assert hm.get("age") == None, "get should return None after delete"
    assert hm.has("age") == False, "has should return False after delete"

    # Test get missing key
    assert hm.get("missing") == None, "get should return None for missing key"

    print("✅ All tests passed!")

