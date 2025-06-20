# How to handle short data messages
## Context
The following [question](https://github.com/CoreTechSecurity/wire-storm/issues/1) was asked to CoreTech:

Referring to the message structure here:
```
0               1               2               3
0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| MAGIC 0xCC    | PADDING       | LENGTH                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| PADDING                                                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| DATA ...................................................... |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7
```
Will there ever be a case where the length of the data section is less than that specified in the length field in the header?

Should such a packet be dropped?

Without giving too much away, should there be recovery mechanisms in place for the "less than" scenario?

### Response
The response was:

This is a good question - and one we have intentionally left ambiguous for candidates when solving the challenge.

It is up to you if and how you handle this case, with no right or wrong answers.

Hint: the tests do not cover this scenario, but there is nothing to prevent you adding more.

The program **had** to handle situtions where the data exceeded length expected. We can account for this by checking the validity of subsequent bytes as the next CTMP header, and recovering appropriately with backtracking.

In the case of undershot message length, there are a series of ways in which we can recover, some more simple than others

## Decision
Guaranteed read up to specified length in header. If the next bytes are not valid, move forward only until we empty the TCP buffer or encounter a valid header. Drop the invalid bytes.

## Alternative
Implement backtracking for past `0xCC` magic bytes to see if there was another header present due to early termination of the `data` portion of the CTMP.

This raised issues of complexity - what if there were multiple "valid headers" in the expected data segment? We would need full context to be able to complete a maximum-length parse.

From a client perspective, the most deterministic behaviour we can provide, and the most obvious behaviour, is to fail fast and move forwards. This also guarantees on our end we don't waste resources parsing bad messages.
