from basemap import BaseHashMap

class LinearHashMap(BaseHashMap):
    def __init__(self, capacity=8):
        ## List of key value pairs
        self.pairs = []

    def set(self, key, value):
        for (i, (k, _)) in enumerate(self.pairs):
            if key == k:
                self.pairs[i] = (key, value)
                return
        self.pairs.append((key, value))

    def has(self, key):
        for (k, _) in self.pairs:
            if key == k:
                return True
        return False
    
    def get(self, key):
        for (k, value) in self.pairs:
            if key == k:
                return value
        return None
        
    def delete(self, key):
        to_delete_idx = -1
        for (i, (k, _)) in enumerate(self.pairs):
            if k == key:
                to_delete_idx = i
            
        if to_delete_idx >= 0:
            self.pairs.pop(to_delete_idx)


if __name__ == "__main__":
    hm = LinearHashMap()

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
