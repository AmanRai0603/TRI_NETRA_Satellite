/*
 * adcs_fsw.h — what the flight software exports, and the only way the
 * platform calls into it.
 *
 * The loop engine (SILS), the target's scheduler (PIL, OILS, HILS) and the
 * flight OBC's own executive all drive the flight software through these
 * functions and nothing else. A customer's flight software that implements
 * this header runs in SILS unchanged; that is the contract a customer build
 * is accepted against.
 *
 * TICK MODEL
 *   adcs_fsw_init() once, with the configuration blob generated from the
 *   module descriptors of the parts fitted (SPEC.md §7.5). Then
 *   adcs_fsw_step(now) at the configured rate. Inside a step the flight
 *   software reads its sensors, estimates, allocates and commands through
 *   adcs_hal.h. A step never blocks and never allocates.
 *
 * DETERMINISM
 *   Given the same config blob, the same sequence of now values and the same
 *   bytes returned by the HAL, a step produces the same bytes out. SILS
 *   relies on it to make runs reproducible from their manifest; the parity
 *   ledger relies on it to compare rungs.
 */
#ifndef ADCS_FSW_H
#define ADCS_FSW_H

#include <stddef.h>
#include <stdint.h>

#include "adcs_hal.h"

#ifdef __cplusplus
extern "C" {
#endif

#define ADCS_FSW_ABI_VERSION 1u

typedef struct {
    uint32_t abi_version;       /* must equal ADCS_FSW_ABI_VERSION              */
    const uint8_t *config_blob; /* "adcs-fswcfg/1", generated, CRC-32 protected */
    size_t config_len;
    uint64_t start_ns;          /* adcs_hal_time_ns() at init                   */
} adcs_fsw_init_t;

/* 0 on success. A non-zero return names what was refused in telemetry; the
 * platform stops the run and reports it by name, never substitutes. */
int32_t adcs_fsw_init(const adcs_fsw_init_t *init);

/* One tick. 0 on success; negative on an internal fault the flight software
 * could not contain (the run records it and continues ticking). */
int32_t adcs_fsw_step(uint64_t now_ns);

/* A telecommand, in the packet layout of fsw/tc/. Mode changes, targets,
 * parameter loads. Returns 0 when accepted. */
int32_t adcs_fsw_command(const uint8_t *tc, size_t len);

/* The flight software's own view, for display and the parity ledger ONLY.
 * The platform never feeds these values back into control. On a flight OBC
 * this is served from telemetry, not from memory. */
typedef struct {
    uint64_t t_ns;
    float q_est[4];        /* body from ECI, scalar first                    */
    float w_est[3];        /* rad/s, body                                    */
    float b_est[3];        /* T, body                                        */
    float h_int[3];        /* N.m.s, stored momentum (wheels + rings), body  */
    float m_cmd[3];        /* A.m^2, commanded dipole, body                  */
    float u_cmd[8];        /* per-actuator command in its own unit           */
    uint16_t mode;         /* the mode enumeration in fsw/tm/modes.h         */
    uint16_t faults;       /* bit set of active faults                       */
} adcs_fsw_state_t;

int32_t adcs_fsw_peek(adcs_fsw_state_t *out);

/* An immutable identifier of this build: the hash every run manifest records.
 * Two runs with different build ids are never compared as the same software. */
const char *adcs_fsw_build_id(void);

#ifdef __cplusplus
}
#endif

#endif /* ADCS_FSW_H */
