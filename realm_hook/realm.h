#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Get the required length of first packet.
uint32_t realm_first_pkt_len();

// Get the index of the selected remote peer.
//
// Remote peers are defined in `remote`(default) and `extra_remotes`(extended),
// where there should be at least 1 remote peer whose idx is 0.
//
// idx < 0 means **ban**.
// idx = 0 means **default**.
int32_t realm_decide_remote_idx(int32_t max_remote_idx, const char *pkt);



#ifdef __cplusplus
}
#endif
