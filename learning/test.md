# Proof that the median minimizes the number of moves

Let the sorted array be

\[
a_1 \le a_2 \le \cdots \le a_n
\]

Suppose we choose a target value \(x\), and change every element to \(x\).

## Step 1: Cost for a fixed target

Changing \(a_i\) to \(x\) requires exactly

\[
|a_i - x|
\]

moves, because each move changes an element by \(1\).

So the total number of moves is

\[
f(x) = \sum\_{i=1}^{n} |a_i - x|
\]

Thus, the problem becomes: **find the value of \(x\) that minimizes \(f(x)\).**

---

## Step 2: How the cost changes when \(x\) increases by 1

Compare \(f(x)\) and \(f(x+1)\).

- If \(a_i \le x\), then its distance to the target increases by 1.
- If \(a_i > x\), then its distance decreases by 1.

Let:

- \(L(x)\) = number of elements \(\le x\)
- \(R(x)\) = number of elements \(> x\)

Then

\[
f(x+1) - f(x) = L(x) - R(x)
\]

This tells us:

- if there are more elements on the right, increasing \(x\) helps
- if there are more elements on the left, increasing \(x\) hurts

So the minimum happens when the counts are balanced.

---

## Step 3: Odd number of elements

If \(n = 2k+1\), the median is \(a\_{k+1}\).

### Case 1: \(x < a\_{k+1}\)

Then at most \(k\) elements are \(\le x\), and at least \(k+1\) are \(> x\). So

\[
L(x) - R(x) < 0
\]

which means

\[
f(x+1) < f(x)
\]

So the cost decreases as \(x\) moves toward the median from the left.

### Case 2: \(x \ge a\_{k+1}\)

Then at least \(k+1\) elements are \(\le x\), so

\[
L(x) - R(x) > 0
\]

which means

\[
f(x+1) > f(x)
\]

So the cost increases after passing the median.

Therefore, the minimum is achieved at the median \(a\_{k+1}\).

---

## Step 4: Even number of elements

If \(n = 2k\), the two middle elements are \(a*k\) and \(a*{k+1}\).

For any integer \(x\) such that

\[
a*k \le x < a*{k+1}
\]

there are exactly \(k\) elements on each side, so

\[
L(x) = R(x) = k
\]

Hence

\[
f(x+1) - f(x) = 0
\]

So the cost is constant across the interval between the two middle elements.

That means **any value between the two middle values is optimal**. In particular, either median works.

---

## Conclusion

The function

\[
f(x) = \sum\_{i=1}^{n} |a_i - x|
\]

is minimized when \(x\) is a median of the array.

Therefore, the minimum number of moves is obtained by:

1. sorting the array,
2. choosing the median,
3. summing the absolute differences to that median.

∎
