# Work in progress


# Lock-free elimination back-off stack

A normal lock-free (Treiber) stack linearizes concurrent access through a single
atomic `head` pointer which `put` and `pop` operations loop on trying to
compare-and-swap it. Usually one uses exponential backoff to circumvent
contention. This results in a single sequential bottleneck and a lot of cache
coherence traffic.

A [lock-free elimination back-off
stack](https://people.csail.mit.edu/shanir/publications/Lock_Free.pdf) wraps
such a lock-free (Treiber) stack, but instead of simply exponentially backing
off on compare-and-swap failures, it uses something called an
`EliminationArray`. Each slot within such an `EliminationArray` enables a thread
executing a `put` operation to hand its item over to a thread executing a `pop`
operation.

The result is a lock-free stack that is both _linearizable_ and _parallel_.
