# Deep Dive into HashMaps

**Understanding the data structure that powers dictionaries, maps, and associative arrays**

HashMaps are one of the most fundamental and powerful data structures in computer science. In this deep dive, we'll explore how they work, why they're so fast, and how to implement them from scratch.

---

## What is a HashMap?

A **HashMap** (also called a **Hash Table**, **dictionary**, or **associative array**) is a data structure that stores **key-value pairs** and allows you to:

1. **Store** a value with a key
2. **Retrieve** a value by its key
3. **Check** if a key exists
4. **Delete** a key-value pair

All in **O(1) average time** (constant time - very fast!)

### Real-World Analogy

Think of it as a super-fast phone book! You say: _"Hey, add John Smith's address as Peyton Street"_ → `("John Smith", "Peyton Street")`. Next time you ask _"What's John Smith's address?"_, it instantly returns the address!

### Without a HashMap (Slow)

```python
# Linear search through a list
addresses = [
    ("alice", "G Street"),
    ("alex", "M Street"),
    ("john", "Peyton Street")
]

# To find John's address:
# - Check alice? No
# - Check alex? No
# - Check john? Yes! → "Peyton Street"
```

**Time complexity: O(n)** - highly inefficient for large datasets!

### With a HashMap (Fast)

```python
# Direct access using hash function
addresses = {
    "alice": "G Street",
    "alex": "M Street",
    "john": "Peyton Street"
}

# To find John's address:
addresses["john"]  # → Instant! O(1)
```

---

## The Magic: Hash Functions

The secret sauce is the **hash function** - it converts a key into an array index!

```
hash("name") → 0
hash("age")  → 1
hash("city") → 2
```

**Wait, arrays again?** Yes! At the end of the day, computers are just long contiguous arrays of memory. Arrays are everywhere!

### Core Operations

| Operation         | What It Does                  | Time |
| ----------------- | ----------------------------- | ---- |
| `set(key, value)` | Store/update a key-value pair | O(1) |
| `get(key)`        | Retrieve value for a key      | O(1) |
| `has(key)`        | Check if key exists           | O(1) |
| `delete(key)`     | Remove a key-value pair       | O(1) |

---

## The Collision Problem

Before we discuss implementations, there's a critical problem: **What if two keys hash to the same index?**

```
hash("harris") → 1
hash("leonard") → 1  ← Collision!
```

Do we override? Delete the old key? Neither makes sense - they're different keys!

This is where **collision resolution strategies** come in. Let's explore three fundamental approaches:

---

## Strategy 1: Linear Search (Naive)

### The Idea

**No hashing at all!** Just store key-value pairs in a list and search through them sequentially.

### Memory Layout

```
┌─────────────────────────────────────┐
│ pairs = [                           │
│   ("name", "Alice"),                │
│   ("age", 25),                      │
│   ("city", "NYC"),                  │
│   ("job", "Engineer")               │
│ ]                                   │
└─────────────────────────────────────┘
```

### Example: Finding "city"

```python
# Linear search through all pairs
for key, value in pairs:
    if key == "city":
        return value  # Found after 3 comparisons!

# Steps:
# - Check "name"? No
# - Check "age"? No
# - Check "city"? YES! → return "NYC"
```

### Performance

| Operation  | Time Complexity | Explanation                  |
| ---------- | --------------- | ---------------------------- |
| **Get**    | O(n)            | Search through all pairs     |
| **Set**    | O(n)            | Check if exists, then append |
| **Delete** | O(n)            | Find and remove              |
| **Space**  | O(n)            | Just the pairs, no overhead  |

### When to Use

**Why would anyone use this?**

For small hashmaps (< 10-20 items), linear search is actually **more efficient** due to:

- ✅ **CPU cache locality** - all data in one contiguous block
- ✅ **Simplicity** - no hash function overhead
- ✅ **No wasted space** - no empty buckets

This is **exactly what your WASM compiler uses!** Perfect for small objects in a compiler.

---

## Strategy 2: Separate Chaining

### The Idea

**Hash first, then search a small list.** Instead of searching the entire list, hash the key to find its "bucket," then search only within that bucket.

If two keys collide (hash to the same bucket), append them to a list at that bucket.

### Memory Layout

```
┌─────────────────────────────────────┐
│ buckets = [                         │
│   [],              # bucket 0       │
│   [],              # bucket 1       │
│   [],              # bucket 2       │
│   [],              # bucket 3       │
│ ]                                   │
└─────────────────────────────────────┘
```

### Example: Collision Handling

Let's insert: `("a", 1)`, `("b", 2)`, `("c", 3)` where "a" and "b" collide!

```
hash("a") → 0
hash("b") → 0  ← Collision!
hash("c") → 1

Result:
┌─────────────────────────────────────┐
│ buckets = [                         │
│   [("a", 1), ("b", 2)],  # bucket 0 │
│   [("c", 3)],            # bucket 1 │
│   [],                    # bucket 2 │
│   [],                    # bucket 3 │
│ ]                                   │
└─────────────────────────────────────┘
```

### How to Find "b"

```python
# 1. Hash to find bucket
bucket_idx = hash("b") % 4  # → 0

# 2. Search only bucket 0 (2 items)
for key, value in buckets[0]:
    if key == "b":
        return value  # Found!
```

### Why It's Faster

```
Linear search: Search n items
Chaining:      Search ~n/capacity items per bucket

If capacity = n/2:
  Each bucket has ~2 items on average
  O(n) → O(2) = O(1)! 🎉
```

### Load Factor

```
load_factor = size / capacity

Examples:
- 10 items, 8 buckets:   10/8 = 1.25 (some buckets have 2+ items)
- 100 items, 200 buckets: 100/200 = 0.5 (most buckets have 0-1 items)

Rule: Keep load factor < 0.75 for good performance
When α > 0.75, double capacity and rehash all items.
```

### Performance

| Operation  | Average Case | Worst Case | Explanation                |
| ---------- | ------------ | ---------- | -------------------------- |
| **Get**    | O(1)         | O(n)       | Hash + search small bucket |
| **Set**    | O(1)         | O(n)       | Hash + bucket append       |
| **Delete** | O(1)         | O(n)       | Hash + bucket search       |
| **Space**  | O(n + m)     | O(n + m)   | n items + m buckets        |

### Pros & Cons

✅ **Fast O(1) operations** (average case)  
✅ **Easy to implement** - straightforward logic  
✅ **Works for any size** - can exceed 100% load  
✅ **Easy deletion** - just remove from list

❌ **Extra memory** for bucket array  
❌ **Poor cache locality** - chasing pointers across memory

---

## Strategy 3: Open Addressing (Linear Probing)

### The Idea

**Store everything in ONE flat array.** On collision, probe (search) for the next empty slot. No chaining, no linked lists - just one array with a clever search strategy!

### Example: Insertion with Collision

Let's insert: `("a", 1)`, `("b", 2)`, `("c", 3)` into a table of 5 slots.

**Step 1:** Insert "a"

```
hash("a") → 0

table = [
    ("a", 1),
    None,
    None,
    None,
    None
]
```

**Step 2:** Insert "b" - Collision!

```
hash("b") → 0  ← Same as "a"!

Probe: slot 0 occupied → try slot 1 → empty! Insert here.

table = [
    ("a", 1),
    ("b", 2),  ← probed to next slot
    None,
    None,
    None
]
```

**Step 3:** Insert "c"

```
hash("c") → 0  ← Collision again!

Probe: slot 0 occupied → slot 1 occupied → slot 2 empty! Insert here.

table = [
    ("a", 1),
    ("b", 2),
    ("c", 3),  ← probed two slots
    None,
    None
]
```

### How to Search

To find "b":

```python
idx = hash("b") % 5  # → 0

# Check slot 0: key is "a" (not "b") → probe
# Check slot 1: key is "b" → Found! Return 2
```

### The Deletion Problem

**Critical issue:** You can't just set a deleted slot to `None`!

```
table = [
    ("a", 1),
    ("b", 2),
    ("c", 3),
    None,
    None
]

# Delete "b"
table[1] = None  # WRONG!

# Now try to find "c":
# hash("c") → 0
# slot 0: "a" (not "c") → probe
# slot 1: None (empty!) → STOP! "c" not found
# But "c" is at slot 2! We can't reach it!
```

### Solution: Tombstones

Use a special **DELETED** marker that:

- Counts as "occupied" during search (keep probing)
- Can be replaced during insertion (treat as empty)

```
table = [
    ("a", 1),
    DELETED,   ← Can be replaced, but search continues
    ("c", 3),
    None,
    None
]

# Now finding "c" works:
# hash("c") → 0
# slot 0: "a" → probe
# slot 1: DELETED → keep probing!
# slot 2: "c" → Found! ✓
```

### Probing Strategies

**Probing** = which index to try next on collision

**1. Linear Probing** (what we showed)

```
Sequence: i, i+1, i+2, i+3, ...

✅ Simple to implement
✅ Excellent cache locality (sequential memory access)
❌ Primary clustering (consecutive occupied slots)
```

**2. Quadratic Probing**

```
Sequence: i, i+1², i+2², i+3², ... = i, i+1, i+4, i+9, i+16, ...

✅ Spreads out more
✅ Reduces primary clustering
❌ Secondary clustering (same hash → same sequence)
```

**3. Double Hashing**

```
Sequence: i, i+h₂, i+2h₂, i+3h₂, ...
(h₂ = second hash function)

✅ Best distribution
✅ No clustering
❌ More computation
❌ Worse cache locality
```

### Load Factor

**Critical:** Must keep α < 0.5-0.7 for good performance!

```
α = size / capacity

At 50% full: average probe length ~1.5
At 75% full: average probe length ~4
At 90% full: average probe length ~10+
```

Higher load → more collisions → longer probe sequences → slower!

### Performance

| Operation  | Average Case | Worst Case | Explanation              |
| ---------- | ------------ | ---------- | ------------------------ |
| **Get**    | O(1)         | O(n)       | If load < 0.5            |
| **Set**    | O(1)         | O(n)       | If load < 0.5            |
| **Delete** | O(1)         | O(n)       | With tombstones          |
| **Space**  | O(m)         | O(m)       | Need empty slots (m > n) |

### Pros & Cons

✅ **Excellent cache locality** - one contiguous array  
✅ **No pointer overhead** - compact memory  
✅ **Fast on modern CPUs** - cache-friendly

❌ **Clustering** - consecutive occupied slots (see below)  
❌ **Tombstone accumulation** - deleted slots waste space  
❌ **Must keep low load factor** - need extra capacity  
❌ **Can fill up completely** - requires resize

<details>
<summary><strong>🔍 Advanced Topic: What is Clustering?</strong></summary>

## Understanding Clustering

**Clustering** occurs when occupied slots bunch together into long consecutive runs, degrading hash table performance.

### Visual Comparison

**Good distribution (no clustering):**

```
[_, X, _, _, X, _, X, _]
 ↑     ↑     ↑
spread out evenly
```

**Bad clustering:**

```
[_, X, X, X, X, X, _, _]
   └───cluster───┘
long run of occupied slots
```

---

## Why Clustering is Problematic: The Snowball Effect

Once a cluster forms, it becomes **more likely to grow**! Let's see why with an example.

### Example: Linear Probing with Capacity = 8

**Step 1:** Insert "a"

```
hash("a") = 2
table = [_, _, a, _, _, _, _, _]
            ↑
         slot 2
```

**Step 2:** Insert "b" (collision!)

```
hash("b") = 2  ← Same as "a"!
Probe: slot 2 occupied → try slot 3 → empty!
table = [_, _, a, b, _, _, _, _]
            └─┬─┘
           cluster
```

**Step 3:** Insert "c" (collision!)

```
hash("c") = 2  ← Collision again!
Probe: slot 2 occupied → slot 3 occupied → try slot 4 → empty!
table = [_, _, a, b, c, _, _, _]
            └───┬───┘
             cluster
```

Now the cluster spans slots 2-4.

### The Problem Escalates

**Step 4:** Insert "d" (hashes to 1, NOT in cluster!)

```
hash("d") = 1
Probe: slot 1 empty → insert!
table = [_, d, a, b, c, _, _, _]
         └─────┬─────┘
           cluster grew!
```

The cluster absorbed slot 1! Now it spans 1-4.

**Step 5:** Insert "e" (hashes to 5, also NOT in cluster!)

```
hash("e") = 5
Probe: slot 5 empty → insert!
table = [_, d, a, b, c, e, _, _]
         └─────────┬─────────┘
           cluster: slots 1-5!
```

The cluster ate slots that hashed to 1 AND 5! It's now 5 consecutive slots.

### Why Clusters Grow Faster

**Snowball Effect:** Larger clusters have higher collision probability!

```
Cluster size k → probability of collision = k / capacity

Example with capacity = 8:
- Cluster size 1: 1/8 = 12.5% collision chance
- Cluster size 3: 3/8 = 37.5% collision chance
- Cluster size 5: 5/8 = 62.5% collision chance
```

Bigger clusters → more collisions → clusters grow even faster! ❄️

---

## Mathematical Analysis

### Expected Probe Length

Without clustering (uniform distribution):

```
E[probes] = 1 / (1 - α)

where α = load factor = n / capacity
```

**Derivation:**

- α = probability a random slot is occupied
- 1 - α = probability a slot is empty (success!)
- E[probes] = 1 / P(success) = 1 / (1 - α)

**Examples:**

```
α = 0.5 (50% full): E = 1/(1-0.5) = 2.0 probes
α = 0.75 (75% full): E = 1/(1-0.75) = 4.0 probes
α = 0.9 (90% full): E = 1/(1-0.9) = 10.0 probes
```

### Impact of Clustering

Clustering **increases** average probe length beyond the theoretical minimum!

**Concrete Example: 4 items, capacity 8**

**Scenario A: No Clustering (Spread Out)**

```
table = [X, _, X, _, X, _, X, _]
         0  1  2  3  4  5  6  7
```

Calculate expected probes for random insertions:

```
Hash to slot 0: occupied → probe to 1 (empty) → 2 probes
Hash to slot 1: empty → 1 probe
Hash to slot 2: occupied → probe to 3 (empty) → 2 probes
Hash to slot 3: empty → 1 probe
Hash to slot 4: occupied → probe to 5 (empty) → 2 probes
Hash to slot 5: empty → 1 probe
Hash to slot 6: occupied → probe to 7 (empty) → 2 probes
Hash to slot 7: empty → 1 probe

E[probes] = (2 + 1 + 2 + 1 + 2 + 1 + 2 + 1) / 8
          = 12 / 8
          = 1.5 probes
```

**Scenario B: With Clustering**

```
table = [X, X, X, X, _, _, _, _]
         0  1  2  3  4  5  6  7
         └───cluster───┘
```

Calculate expected probes:

```
Hash to slot 0: occupied → 1 → 2 → 3 → 4 (empty) → 5 probes
Hash to slot 1: occupied → 2 → 3 → 4 (empty) → 4 probes
Hash to slot 2: occupied → 3 → 4 (empty) → 3 probes
Hash to slot 3: occupied → 4 (empty) → 2 probes
Hash to slot 4: empty → 1 probe
Hash to slot 5: empty → 1 probe
Hash to slot 6: empty → 1 probe
Hash to slot 7: empty → 1 probe

E[probes] = (5 + 4 + 3 + 2 + 1 + 1 + 1 + 1) / 8
          = 18 / 8
          = 2.25 probes
```

**Result:** 1.5 vs 2.25 probes — that's a **50% increase** due to clustering! 📈

### Key Insight

> **"A cluster acts like one giant slot that guarantees multiple probes"**

The larger the cluster, the more probes required on average to traverse it.

</details>

---

## Strategy Comparison

Here's a side-by-side comparison to help you choose the right implementation:

| Aspect                 | Linear Search           | Separate Chaining               | Open Addressing                |
| ---------------------- | ----------------------- | ------------------------------- | ------------------------------ |
| **Data Structure**     | Simple array of pairs   | Array of buckets (linked lists) | Single flat array              |
| **Get Time**           | O(n)                    | O(1) average                    | O(1) average                   |
| **Set Time**           | O(n)                    | O(1) average                    | O(1) average                   |
| **Delete Time**        | O(n)                    | O(1) average                    | O(1) average (with tombstones) |
| **Space Complexity**   | O(n)                    | O(n + m)                        | O(m) where m > n               |
| **Cache Locality**     | ✅ Excellent            | ❌ Poor (pointer chasing)       | ✅ Excellent                   |
| **Load Factor**        | N/A (always 100%)       | Can exceed 100%                 | Must stay < 70%                |
| **Collision Handling** | N/A                     | Chain to same bucket            | Probe to next slot             |
| **Deletion**           | Simple removal          | Simple removal                  | Requires tombstones            |
| **Memory Overhead**    | None                    | Pointers + buckets              | Empty slots                    |
| **Best For**           | Small maps (< 20 items) | General purpose                 | Cache-sensitive applications   |
| **Worst Case**         | O(n) all operations     | O(n) if all keys collide        | O(n) if clustering occurs      |

### When to Use Each Strategy

**Linear Search:**

- Small hashmaps (< 10-20 items)
- Memory-constrained environments
- Simple, predictable performance

**Separate Chaining:**

- General-purpose hashmaps
- Unknown or variable dataset sizes
- When deletions are frequent
- Python's `dict`, Java's `HashMap`

**Open Addressing:**

- Cache-sensitive applications
- Memory-constrained (no pointer overhead)
- Predictable max size
- Rust's `HashMap`, C++'s `unordered_map`

---

## Quiz: Test Your Understanding

### Question 1: Load Factor

What is the load factor of a hash table with 75 items and a capacity of 100?

<details>
<summary>Click to see answer</summary>

**Answer:** α = 75/100 = 0.75 (or 75%)

This is at the threshold where most implementations resize the table!

</details>

### Question 2: Collision Resolution

In a hash table using **separate chaining**, two keys hash to the same bucket. What happens?

<details>
<summary>Click to see answer</summary>

**Answer:** Both key-value pairs are stored in a list (chain) at that bucket.

When you search for a key, you:

1. Hash the key to find the bucket
2. Search through the list at that bucket

This is why it's called "chaining" - you chain multiple items together!

</details>

### Question 3: Time Complexity

Why is the average-case time complexity of hash table operations O(1), but worst-case is O(n)?

<details>
<summary>Click to see answer</summary>

**Answer:**

**Average case O(1):** With a good hash function and proper load factor, keys are distributed evenly. Each bucket/slot has only a few items (ideally 0-1), so lookup is constant time.

**Worst case O(n):** If all keys hash to the same bucket (terrible hash function) or if clustering occurs, you end up doing a linear search through all n items.

Example worst case:

- Separate chaining: all n keys in one bucket → search n items
- Open addressing: all n keys in one cluster → probe n times
</details>

### Question 4: Tombstones

Why can't you simply set a deleted slot to `None` in open addressing with linear probing?

<details>
<summary>Click to see answer</summary>

**Answer:** Setting a slot to `None` breaks the probe sequence!

Example:

```
Insert "a" → slot 0
Insert "b" (collision) → probe to slot 1
Insert "c" (collision) → probe to slot 2

table = [a, b, c, _, _]

Delete "b" → table[1] = None

table = [a, None, c, _, _]

Now search for "c":
- hash("c") = 0
- slot 0: "a" (not "c") → probe
- slot 1: None (empty!) → STOP! "c" not found
```

We can't reach "c" anymore! The `None` at slot 1 terminates the probe sequence.

**Solution:** Use a **DELETED** tombstone marker that:

- Tells search to keep probing (not empty)
- Tells insert it can reuse this slot (not occupied)
</details>

### Question 5: Performance Analysis

Given a hash table with open addressing at 90% load factor (α = 0.9), what is the expected number of probes?

<details>
<summary>Click to see answer</summary>

**Answer:** E[probes] = 1 / (1 - α) = 1 / (1 - 0.9) = 1 / 0.1 = **10 probes**

This is why we keep load factor below 0.7-0.75 in practice! At 90% full:

- Very high collision probability
- Long probe sequences
- Significant performance degradation

Compare to α = 0.5: E[probes] = 1 / (1 - 0.5) = 2 probes

</details>

### Question 6: Cache Locality

Why does open addressing generally have better cache performance than separate chaining?

<details>
<summary>Click to see answer</summary>

**Answer:** **Memory layout matters for CPU caches!**

**Open addressing:**

- All data in one contiguous array
- Probing accesses sequential memory locations
- High probability data is already in CPU cache
- Cache-friendly! ✅

**Separate chaining:**

- Buckets are an array, but each bucket points to a separate list
- Following pointers "jumps" to different memory locations
- Each jump might cause a cache miss
- Cache-unfriendly! ❌

Modern CPUs are **much faster** at accessing sequential memory than random memory locations. This is why open addressing can outperform chaining despite similar algorithmic complexity.

</details>

---

## Mathematical Deep Dive Questions

These questions require rigorous thinking and mathematical analysis. Work through them carefully!

### Question 7: Collision Probability

You have a hash table with **capacity = 100** using separate chaining. You insert **20 random keys**. What is the probability that **at least two keys** hash to the same bucket?

**Hint:** Think about the birthday paradox!

<details>
<summary>Click to see solution</summary>

**Solution:**

It's easier to calculate the probability that **no collisions occur**, then subtract from 1.

**Probability of no collisions:**

- First key: can go anywhere → P = 100/100 = 1
- Second key: must avoid 1 occupied bucket → P = 99/100
- Third key: must avoid 2 occupied buckets → P = 98/100
- ...
- 20th key: must avoid 19 occupied buckets → P = 81/100

```
P(no collision) = (100/100) × (99/100) × (98/100) × ... × (81/100)
                = 100! / (80! × 100^20)
                ≈ 0.1318
```

Therefore:
```
P(at least one collision) = 1 - P(no collision)
                          = 1 - 0.1318
                          = 0.8682 = 86.82%
```

**Answer: ~87% chance of at least one collision!**

This is the **birthday paradox** applied to hash tables - collisions are much more common than you'd intuitively expect!

</details>

### Question 8: Clustering Analysis

Consider an open addressing table with capacity = 8 and load factor α = 0.5 (4 items).

**Part A:** Calculate the expected probe length if items are **uniformly distributed** (no clustering).

**Part B:** Calculate the expected probe length if all 4 items form a **contiguous cluster** starting at index 0.

**Part C:** What is the percentage increase in expected probes due to clustering?

<details>
<summary>Click to see solution</summary>

**Part A: Uniform Distribution**

Table: `[X, _, X, _, X, _, X, _]`

For a random insertion (uniform hash):
```
P(hash to slot i) = 1/8 for all i

Expected probes:
- Hash to empty slot (slots 1,3,5,7): 1 probe × 4/8 = 0.5
- Hash to occupied slot (slots 0,2,4,6): 2 probes × 4/8 = 1.0
  (must probe once to find it's occupied, once more to find empty)

E[probes] = 0.5 + 1.0 = 1.5 probes
```

Alternatively using the formula: E[probes] = 1/(1-α) = 1/(1-0.5) = 2 probes (for finding a key, but for insertion it's 1.5 average)

**Part B: Clustered Distribution**

Table: `[X, X, X, X, _, _, _, _]`

For a random insertion:
```
Hash to slot 0: probe 0→1→2→3→4 (empty) = 5 probes
Hash to slot 1: probe 1→2→3→4 (empty) = 4 probes  
Hash to slot 2: probe 2→3→4 (empty) = 3 probes
Hash to slot 3: probe 3→4 (empty) = 2 probes
Hash to slot 4: probe 4 (empty) = 1 probe
Hash to slot 5: probe 5 (empty) = 1 probe
Hash to slot 6: probe 6 (empty) = 1 probe
Hash to slot 7: probe 7 (empty) = 1 probe

E[probes] = (5 + 4 + 3 + 2 + 1 + 1 + 1 + 1) / 8
          = 18 / 8
          = 2.25 probes
```

**Part C: Percentage Increase**

```
Increase = (2.25 - 1.5) / 1.5 × 100%
         = 0.75 / 1.5 × 100%
         = 50%
```

**Answer: Clustering causes a 50% increase in expected probe length!** This demonstrates why clustering is such a critical problem.

</details>

### Question 9: Optimal Resize Threshold

You're designing a hash table with open addressing. Each resize operation costs **O(n)** to rehash all items. Waiting too long increases probe length, but resizing too often wastes time.

Given that E[probes] = 1/(1-α), at what load factor α should you resize to **minimize the total cost** of operations?

**Assume:** You start at capacity C and will eventually reach 2C items (so you need exactly one resize).

<details>
<summary>Click to see solution</summary>

**Solution:**

Let's denote the resize threshold as α_resize.

**Phase 1:** Insert from 0 to α_resize × C items
- Average load factor: α_resize / 2
- Average probe length: 1 / (1 - α_resize/2)
- Total cost: (α_resize × C) × [1 / (1 - α_resize/2)]

**Resize cost:** O(α_resize × C)

**Phase 2:** Insert from α_resize × C to 2C items into a table of size 2C
- Items inserted: 2C - α_resize × C = C(2 - α_resize)
- Average load factor during phase 2: ~α_resize/2
- Average probe length: 1 / (1 - α_resize/2)
- Total cost: C(2 - α_resize) × [1 / (1 - α_resize/2)]

**Total cost:**
```
Cost = Phase1 + Resize + Phase2
     = α_resize × C × [1/(1 - α_resize/2)] 
       + α_resize × C 
       + C(2 - α_resize) × [1/(1 - α_resize/2)]
```

Taking the derivative with respect to α_resize and setting to 0 yields:

**α_resize ≈ 0.7 - 0.75**

This is why most implementations use **0.75** as the load factor threshold! It's not arbitrary - it's mathematically optimal for balancing resize cost vs probe length.

**Answer: α ≈ 0.75 is optimal**

</details>

### Question 10: Expected Collisions in Separate Chaining

You have a hash table with **m buckets** using separate chaining. You insert **n items** with a perfect uniform hash function.

**Derive:** What is the expected number of buckets that contain **exactly 2 items**?

<details>
<summary>Click to see solution</summary>

**Solution:**

For a specific bucket, we need to calculate P(exactly 2 items land in this bucket).

Each item has probability 1/m of hashing to this bucket.

Using the binomial distribution:
```
P(exactly k items in bucket) = C(n,k) × (1/m)^k × (1 - 1/m)^(n-k)

For k = 2:
P(exactly 2 items) = C(n,2) × (1/m)^2 × (1 - 1/m)^(n-2)
                   = [n(n-1)/2] × (1/m)^2 × (1 - 1/m)^(n-2)
```

Since there are **m buckets** total:

```
E[buckets with exactly 2 items] = m × P(exactly 2 items in one bucket)
                                  = m × [n(n-1)/2] × (1/m)^2 × (1 - 1/m)^(n-2)
                                  = [n(n-1)/2m] × (1 - 1/m)^(n-2)
```

**Special case:** When n = m (load factor = 1):

```
E[buckets with 2 items] = [m(m-1)/2m] × (1 - 1/m)^(m-2)
                         ≈ (m/2) × (1 - 1/m)^m
                         ≈ (m/2) × (1/e)    [as m → ∞]
                         ≈ m/(2e)
                         ≈ 0.184m
```

**Answer: For load factor = 1, approximately 18.4% of buckets will have exactly 2 items.**

The rest will have 0, 1, or 3+ items following a Poisson distribution!

</details>

---

## Implementation Challenges

Ready to code? Try implementing each strategy yourself!

### Challenge 1: Linear Search HashMap

Implement a simple hashmap using linear search (no hashing).

**Requirements:**

- `set(key, value)` - add or update a key-value pair
- `get(key)` - retrieve value for a key (return None if not found)
- `has(key)` - check if key exists
- `delete(key)` - remove a key-value pair

**Starter code (Python):**

```python
class LinearHashMap:
    def __init__(self):
        self.pairs = []  # List of (key, value) tuples

    def set(self, key, value):
        # TODO: implement
        pass

    def get(self, key):
        # TODO: implement
        pass

    def has(self, key):
        # TODO: implement
        pass

    def delete(self, key):
        # TODO: implement
        pass

# Test it!
hm = LinearHashMap()
hm.set("name", "Alice")
hm.set("age", 25)
print(hm.get("name"))  # Should print "Alice"
print(hm.has("city"))  # Should print False
hm.delete("age")
print(hm.get("age"))   # Should print None
```

<details>
<summary>Solution</summary>

```python
class LinearHashMap:
    def __init__(self):
        self.pairs = []

    def set(self, key, value):
        # Check if key exists - update if so
        for i, (k, v) in enumerate(self.pairs):
            if k == key:
                self.pairs[i] = (key, value)
                return
        # Key doesn't exist - append new pair
        self.pairs.append((key, value))

    def get(self, key):
        for k, v in self.pairs:
            if k == key:
                return v
        return None

    def has(self, key):
        for k, v in self.pairs:
            if k == key:
                return True
        return False

    def delete(self, key):
        for i, (k, v) in enumerate(self.pairs):
            if k == key:
                self.pairs.pop(i)
                return
```

**Time Complexity:** All operations are O(n)

</details>

---

### Challenge 2: Separate Chaining HashMap

Implement a hashmap using separate chaining with a bucket array.

**Requirements:**

- Use a hash function to map keys to buckets
- Handle collisions by chaining items in the same bucket
- Implement `set`, `get`, `has`, `delete`
- Start with capacity = 8

**Starter code (Python):**

```python
class ChainingHashMap:
    def __init__(self, capacity=8):
        self.capacity = capacity
        self.buckets = [[] for _ in range(capacity)]  # List of lists
        self.size = 0

    def _hash(self, key):
        # Simple hash: use Python's built-in hash
        return hash(key) % self.capacity

    def set(self, key, value):
        # TODO: implement
        # Hint: hash key → find bucket → search bucket → update or append
        pass

    def get(self, key):
        # TODO: implement
        pass

    def has(self, key):
        # TODO: implement
        pass

    def delete(self, key):
        # TODO: implement
        pass

# Test it!
hm = ChainingHashMap()
hm.set("a", 1)
hm.set("b", 2)
hm.set("c", 3)
print(hm.get("b"))  # Should print 2
```

<details>
<summary>Solution</summary>

```python
class ChainingHashMap:
    def __init__(self, capacity=8):
        self.capacity = capacity
        self.buckets = [[] for _ in range(capacity)]
        self.size = 0

    def _hash(self, key):
        return hash(key) % self.capacity

    def set(self, key, value):
        bucket_idx = self._hash(key)
        bucket = self.buckets[bucket_idx]

        # Check if key exists in bucket
        for i, (k, v) in enumerate(bucket):
            if k == key:
                bucket[i] = (key, value)  # Update
                return

        # Key doesn't exist - append to bucket
        bucket.append((key, value))
        self.size += 1

    def get(self, key):
        bucket_idx = self._hash(key)
        bucket = self.buckets[bucket_idx]

        for k, v in bucket:
            if k == key:
                return v
        return None

    def has(self, key):
        bucket_idx = self._hash(key)
        bucket = self.buckets[bucket_idx]

        for k, v in bucket:
            if k == key:
                return True
        return False

    def delete(self, key):
        bucket_idx = self._hash(key)
        bucket = self.buckets[bucket_idx]

        for i, (k, v) in enumerate(bucket):
            if k == key:
                bucket.pop(i)
                self.size -= 1
                return
```

**Time Complexity:** O(1) average, O(n) worst case

</details>

---

### Challenge 3: Open Addressing HashMap (Linear Probing)

Implement a hashmap using open addressing with linear probing.

**Requirements:**

- Store key-value pairs in a flat array
- Use linear probing for collision resolution
- Handle deletions with a DELETED tombstone marker
- Implement `set`, `get`, `has`, `delete`
- Keep load factor < 0.7 (resize when needed)

**Starter code (Python):**

```python
DELETED = object()  # Unique tombstone marker

class OpenAddressingHashMap:
    def __init__(self, capacity=8):
        self.capacity = capacity
        self.table = [None] * capacity
        self.size = 0

    def _hash(self, key):
        return hash(key) % self.capacity

    def _load_factor(self):
        return self.size / self.capacity

    def set(self, key, value):
        # TODO: implement
        # Hint: hash key → probe until you find key or empty slot
        # Don't forget to resize if load factor > 0.7!
        pass

    def get(self, key):
        # TODO: implement
        # Hint: probe until you find key, None, or loop back to start
        pass

    def has(self, key):
        # TODO: implement
        pass

    def delete(self, key):
        # TODO: implement
        # Hint: set to DELETED, not None!
        pass

    def _resize(self):
        # TODO: implement
        # Hint: create new table, rehash all items
        pass

# Test it!
hm = OpenAddressingHashMap()
hm.set("a", 1)
hm.set("b", 2)
hm.set("c", 3)
print(hm.get("b"))  # Should print 2
hm.delete("b")
print(hm.get("b"))  # Should print None
```

<details>
<summary>Solution</summary>

```python
DELETED = object()

class OpenAddressingHashMap:
    def __init__(self, capacity=8):
        self.capacity = capacity
        self.table = [None] * capacity
        self.size = 0

    def _hash(self, key):
        return hash(key) % self.capacity

    def _load_factor(self):
        return self.size / self.capacity

    def set(self, key, value):
        if self._load_factor() > 0.7:
            self._resize()

        idx = self._hash(key)
        start_idx = idx

        while True:
            slot = self.table[idx]

            # Empty or deleted slot - insert here
            if slot is None or slot is DELETED:
                self.table[idx] = (key, value)
                self.size += 1
                return

            # Key exists - update
            if slot[0] == key:
                self.table[idx] = (key, value)
                return

            # Probe to next slot
            idx = (idx + 1) % self.capacity

            # Looped back - table is full (shouldn't happen with resize)
            if idx == start_idx:
                raise Exception("Hash table is full!")

    def get(self, key):
        idx = self._hash(key)
        start_idx = idx

        while True:
            slot = self.table[idx]

            # Empty - not found
            if slot is None:
                return None

            # Found key
            if slot is not DELETED and slot[0] == key:
                return slot[1]

            # Probe to next slot (skip DELETED)
            idx = (idx + 1) % self.capacity

            # Looped back - not found
            if idx == start_idx:
                return None

    def has(self, key):
        return self.get(key) is not None

    def delete(self, key):
        idx = self._hash(key)
        start_idx = idx

        while True:
            slot = self.table[idx]

            # Empty - not found
            if slot is None:
                return

            # Found key - mark as deleted
            if slot is not DELETED and slot[0] == key:
                self.table[idx] = DELETED
                self.size -= 1
                return

            # Probe to next slot
            idx = (idx + 1) % self.capacity

            # Looped back - not found
            if idx == start_idx:
                return

    def _resize(self):
        old_table = self.table
        self.capacity *= 2
        self.table = [None] * self.capacity
        self.size = 0

        # Rehash all items
        for slot in old_table:
            if slot is not None and slot is not DELETED:
                key, value = slot
                self.set(key, value)
```

**Time Complexity:** O(1) average, O(n) worst case

**Key Points:**

- Tombstones (DELETED) allow probe sequences to continue
- Resize when load factor > 0.7 to maintain performance
- Linear probing checks consecutive slots
</details>

---

## Bonus Challenge: Analyze Your Code

After implementing each strategy, answer these questions:

1. **What is the actual time complexity** of your implementations?
2. **Run a benchmark:** Insert 10,000 random key-value pairs. Which is fastest? Why?
3. **Memory usage:** Which implementation uses the most memory? Least?
4. **When does each implementation break down?** Try inserting 1 million items.
5. **Collision behavior:** Intentionally create collisions (e.g., all keys hash to 0). How does each strategy handle it?

---

## Congratulations!

You've mastered hashmap fundamentals! You now understand:

✅ How hashmaps provide O(1) average-case operations  
✅ Three fundamental implementation strategies  
✅ The tradeoffs between time, space, and cache locality  
✅ How load factor affects performance  
✅ Why clustering degrades open addressing  
✅ How real-world languages implement hashmaps

**Next steps:**

- Implement these in your WASM compiler!
- Explore advanced techniques: Robin Hood hashing, Cuckoo hashing
- Study how Rust's `HashMap` uses SipHash for security
- Learn about concurrent hashmaps (lock-free, sharded)

Happy hashing! 🎉
