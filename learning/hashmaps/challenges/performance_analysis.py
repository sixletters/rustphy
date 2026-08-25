import chaining
import linear
import open_addressing
import time, tracemalloc
from typing import Callable
from basemap import BaseHashMap
import random


## generate the keys one time
sizes = [1000, 10000, 100000, 1000000]
keys = [str(i) for i in range(max(sizes))]


def bench_scale(mapFactory: Callable[[int], BaseHashMap], sizes: list[int]):
    for n in sizes:
        subset = keys[:n]

        current_map = mapFactory(n * 2)

        ## q: what does tracemalloc actually do?
        tracemalloc.start()
        t0 = time.perf_counter()

        for k in subset:
            current_map.set(k, 1)
        t1 = time.perf_counter()

        ## q: what does get_traced_memory do? and what does this actually trace?
        ## amount of allocated memory
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        print(n, t1-t0)
        print(f"peak is {peak / 1024:.1f} KB")
        print(f"current was {current / 1024:.1f} KB")

def bench_load_factor(mapFactory: Callable[[int], BaseHashMap], map_type: str, load_factor: list[int] = [0.3, 0.5, 0.7, 0.9]):
    ## What is the load factor again?
    ## load factor = size/capacity. Essentially how much load is on the map right now
    ## expected probe is 1/ 1 - a, where a is the load_factor
    n = 10000
    ## fills the map
    for target_lf in load_factor:
        capacity = int(n/target_lf)
        current_map = mapFactory(capacity)
        for k in keys[:n]:
            current_map.set(k, 1)

        ## samples 100 random keys
        lookup_keys = random.sample(keys[:n], 1000)

        t0 = time.perf_counter()
        total_probes = 0
        for k in lookup_keys:
            current_map.get(k)
            ## todo: add in last_probe_count into BaseHashMap?
            total_probes += current_map.last_probe_count
        t1 = time.perf_counter()
        avg_get_time = (t1 - t0) / len(lookup_keys) * 1_000_000  # microseconds
        avg_probes = total_probes / len(lookup_keys)
        theoretical = 1 / (1 - target_lf) if map_type == "open-addressing" else 1 + target_lf / 2
        print(f"lf={target_lf}  avg_get={avg_get_time:.2f}µs  avg_probes={avg_probes:.2f}  theoretical={theoretical:.2f}")

    
# We want to mainly analyze the following first
# 1. What is the actual time complexity of your implementations
# 2. Run a benchmark: insert 10000 random key-value pairs. Which is fastest? Why?
# 3. Memory usage: which implementaiton uses the most memory? least?
# 4. When does each implementatino break down: insert 1 million items
# 5. Collision behaviour -> how do we analyse this?
if __name__ == "__main__":
    pass
    chainingMap = chaining.ChainingHashMap()
    openAddressingMap = open_addressing.OpenAddressingHashMap()
    linearMap = linear.LinearHashMap()

    testMapFactory = {
        "chaining": chaining.ChainingHashMap,
        "open-addressing": open_addressing.OpenAddressingHashMap,
        "linear": linear.LinearHashMap
    }

    linear_sizes = [1000, 10000]
    for mapName, mapFactory in testMapFactory.items():
        s = linear_sizes if mapName == "linear" else sizes
        # bench_scale(lambda x: mapFactory(x), s)
        if mapName == "linear":
            continue
        if mapName == "open-addressing":
            print("open-addressing")
            bench_load_factor(lambda x: open_addressing.OpenAddressingHashMap(capacity=x, auto_resize=False), mapName)
        else:
            print("chaining")
            bench_load_factor(lambda x: mapFactory(x), mapName)
