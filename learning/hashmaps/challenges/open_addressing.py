from basemap import BaseHashMap

DELETED = object()  # Unique tombstone marker, its a constant

class OpenAddressingHashMap(BaseHashMap):
    def __init__(self, capacity=8, auto_resize=True):
        self.capacity = capacity
        self.table = [None] * capacity
        self.size = 0
        self.last_probe_count = 0
        self.auto_resize = auto_resize

    def _hash(self, key):
        return hash(key) % self.capacity

    def _load_factor(self):
        return self.size / self.capacity

    def set(self, key, value):    
        hash_idx = self._hash(key=key)
        ##implement linear probing
        curr_slot = self.table[hash_idx]
        while curr_slot != DELETED and curr_slot != None:
            if curr_slot != DELETED and curr_slot[0] == key:
                ## size does not change so resize not needed
                self.table[hash_idx] = (key, value)
                return
            hash_idx += 1
            hash_idx %= self.capacity
            curr_slot = self.table[hash_idx]

        self.size += 1
        self.table[hash_idx] = (key, value)

        if self.auto_resize and self._load_factor() > 0.7:
            self._resize()

    def get(self, key):
        # probe until you find key, None, or loop back to start
        self.last_probe_count = 0
        hash_idx = self._hash(key=key)
        start = hash_idx

        ## implement linear probing search
        curr_slot = self.table[hash_idx]
        while curr_slot != None:
            self.last_probe_count += 1
            if curr_slot != DELETED and curr_slot[0] == key:
                return curr_slot[1]
            hash_idx += 1
            hash_idx %= self.capacity
            curr_slot = self.table[hash_idx]
            if hash_idx == start:
                break

        return None

    def has(self, key):
        return self.get(key=key) != None

    def delete(self, key):
        # probe until you find key, None, or loop back to start
        hash_idx = self._hash(key=key)
        start = hash_idx

        ## implement linear probing search
        curr_slot = self.table[hash_idx]
        while curr_slot != None:
            if curr_slot != DELETED and curr_slot[0] == key:
                self.table[hash_idx] = DELETED
                self.size -= 1
                return
            hash_idx += 1
            hash_idx %= self.capacity
            curr_slot = self.table[hash_idx]
            if hash_idx == start:
                break

    def _resize(self):
        self.capacity = self.capacity << 2
        self.table, old_table = [None for _ in range(self.capacity)], self.table
        for slot in old_table:
            if slot == DELETED or slot == None:
                continue
            k,v = slot
            self.set(k, v)

if __name__ == "__main__":
    hm = OpenAddressingHashMap(8)

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

    # Test resizing - start with small capacity
    hm2 = OpenAddressingHashMap(capacity=4)
    assert hm2.capacity == 4, "initial capacity should be 4"

    # Insert items to trigger resize (load factor > 0.7 means > 2.8 items)
    hm2.set("a", 1)
    hm2.set("b", 2)
    hm2.set("c", 3)  # This should trigger resize (3/4 = 0.75 > 0.7)

    # After resize, capacity should increase
    assert hm2.capacity > 4, f"capacity should increase after resize, got {hm2.capacity}"

    # All items should still be retrievable
    assert hm2.get("a") == 1, "should retrieve 'a' after resize"
    assert hm2.get("b") == 2, "should retrieve 'b' after resize"
    assert hm2.get("c") == 3, "should retrieve 'c' after resize"

    print("✅ All tests passed!")


