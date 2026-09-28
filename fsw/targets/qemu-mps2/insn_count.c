/* insn_count -- QEMU TCG plugin for soft OILS: the exact number of guest instructions the
 * flight software executes per step on the virtual Cortex-M4F.
 *
 * The OBC firmware brackets adcs_fsw_step with two marker instructions no compiler emits,
 * `mov r9, r9` (0x46C9, start) and `mov r10, r10` (0x46D2, stop). The plugin counts every
 * executed translation block inline and, at each marker, the marker's position inside its
 * block, so the count between the markers is exact and repeatable run to run. At each stop
 * it appends the count (u64, little-endian) to the file given as out=<path>; the engine reads
 * one count per TICK (engine/crates/adcs-fsw-abi/src/link.rs).
 *
 *   qemu-system-arm ... -plugin fsw/build/insn_count.so,out=/tmp/counts.bin
 *
 * The declarations below are the handful of QEMU plugin API v1 entry points used (QEMU >= 6),
 * written from the API documentation, so no QEMU source is copied into this tree.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>

typedef uint64_t qemu_plugin_id_t;
struct qemu_plugin_tb;
struct qemu_plugin_insn;
typedef struct qemu_info_t qemu_info_t;
enum qemu_plugin_op { QEMU_PLUGIN_INLINE_ADD_U64 };
enum qemu_plugin_cb_flags { QEMU_PLUGIN_CB_NO_REGS, QEMU_PLUGIN_CB_R_REGS, QEMU_PLUGIN_CB_RW_REGS };
typedef void (*qemu_plugin_vcpu_tb_trans_cb_t)(qemu_plugin_id_t id, struct qemu_plugin_tb *tb);
typedef void (*qemu_plugin_vcpu_udata_cb_t)(unsigned int vcpu_index, void *userdata);

void qemu_plugin_register_vcpu_tb_trans_cb(qemu_plugin_id_t id, qemu_plugin_vcpu_tb_trans_cb_t cb);
void qemu_plugin_register_vcpu_tb_exec_inline(struct qemu_plugin_tb *tb, enum qemu_plugin_op op, void *ptr, uint64_t imm);
void qemu_plugin_register_vcpu_insn_exec_cb(struct qemu_plugin_insn *insn, qemu_plugin_vcpu_udata_cb_t cb,
                                            enum qemu_plugin_cb_flags flags, void *userdata);
size_t qemu_plugin_tb_n_insns(const struct qemu_plugin_tb *tb);
struct qemu_plugin_insn *qemu_plugin_tb_get_insn(const struct qemu_plugin_tb *tb, size_t idx);
const void *qemu_plugin_insn_data(const struct qemu_plugin_insn *insn);
size_t qemu_plugin_insn_size(const struct qemu_plugin_insn *insn);

__attribute__((visibility("default"))) int qemu_plugin_version = 1;

static uint64_t count;          /* instructions of every translation block entered */
static uint64_t start;
static int fd = -1;

/* userdata of a marker: instructions of its block that come after it (already counted at
 * block entry), in the low bits; bit 63 marks a stop */
#define STOP (1ull << 63)

static void on_marker(unsigned int vcpu, void *ud)
{
    uint64_t u = (uint64_t)(uintptr_t)ud, pos = count - (u & ~STOP);
    (void)vcpu;
    if (!(u & STOP)) { start = pos; return; }
    if (fd >= 0) {
        uint64_t n = pos - start - 1;               /* the instructions strictly between the markers */
        uint8_t b[8]; int k;
        for (k = 0; k < 8; k++) b[k] = (uint8_t)(n >> (8*k));
        if (write(fd, b, 8) != 8) { /* the engine went away */ }
    }
}

static void on_tb(qemu_plugin_id_t id, struct qemu_plugin_tb *tb)
{
    size_t n = qemu_plugin_tb_n_insns(tb), i;
    (void)id;
    qemu_plugin_register_vcpu_tb_exec_inline(tb, QEMU_PLUGIN_INLINE_ADD_U64, &count, n);
    for (i = 0; i < n; i++) {
        struct qemu_plugin_insn *in = qemu_plugin_tb_get_insn(tb, i);
        const uint8_t *d = qemu_plugin_insn_data(in);
        uint16_t op;
        if (qemu_plugin_insn_size(in) != 2) continue;
        op = (uint16_t)(d[0] | (d[1] << 8));
        if (op == 0x46C9u || op == 0x46D2u) {
            uint64_t after = (uint64_t)(n - 1 - i);
            qemu_plugin_register_vcpu_insn_exec_cb(in, on_marker, QEMU_PLUGIN_CB_NO_REGS,
                                                   (void *)(uintptr_t)(after | (op == 0x46D2u ? STOP : 0)));
        }
    }
}

__attribute__((visibility("default")))
int qemu_plugin_install(qemu_plugin_id_t id, const qemu_info_t *info, int argc, char **argv)
{
    int i;
    (void)info;
    for (i = 0; i < argc; i++)
        if (strncmp(argv[i], "out=", 4) == 0) fd = open(argv[i] + 4, O_WRONLY | O_CREAT | O_APPEND, 0644);
    qemu_plugin_register_vcpu_tb_trans_cb(id, on_tb);
    return 0;
}
