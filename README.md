**Work in progress**


# Lock-free elimination back-off stack

A normal lock-free Treiber stack [1] linearizes concurrent access through a
single atomic `head` pointer on which `push` and `pop` operations loop trying to
compare-and-swap it. Usually one uses exponential backoff to circumvent
contention. This results in a single sequential bottleneck and a lot of cache
coherence traffic.

A [lock-free elimination back-off
stack](https://people.csail.mit.edu/shanir/publications/Lock_Free.pdf) wraps
such a lock-free Treiber stack, but instead of simply exponentially backing off
on compare-and-swap failures, it uses something called an `EliminationArray`.
Each slot within such an `EliminationArray` enables a thread executing a `push`
operation to hand its item over to a thread executing a `pop` operation. On
contention a thread tries to *exchange* on a randomly chosen slot within the
`EliminationArray`. On failure of such an *exchange* it loops to the beginning
retrying on the stack again.

The result is a lock-free stack that is both _linearizable_ and _parallel_.


[1] Treiber, R. Kent. Systems programming: Coping with parallelism. New York:
International Business Machines Incorporated, Thomas J. Watson Research Center,
1986.
