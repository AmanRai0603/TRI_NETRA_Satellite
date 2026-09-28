/* adcs_drv.h -- device drivers over adcs_hal.h (fsw/pseudocode/09).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#ifndef ADCS_DRV_H
#define ADCS_DRV_H
#include "adcs_math.h"
#include "adcs_params.h"

typedef struct {
    int mag_ok;   adcs_real B[3];
    int gyro_ok;  adcs_real w[3];
    int sun_ok;   adcs_real sun[3];
    int es_ok;    adcs_real nadir[3];
    int st_ok;    int st_valid[ADCS_MAX_HEADS]; adcs_real q_st[ADCS_MAX_HEADS][4];
    int gps_ok;   adcs_real r[3], v[3];
    adcs_real h[ADCS_MAX_ROTORS], delta[ADCS_MAX_GIMBALS];
} adcs_meas_t;

void adcs_drv_reset(void);
void adcs_drv_read(const adcs_params_t *p, adcs_meas_t *z);
void adcs_drv_write(const adcs_params_t *p, const adcs_real m_body[3], const adcs_real cmd_r[ADCS_MAX_ROTORS],
                    const adcs_real cmd_g[ADCS_MAX_GIMBALS], const adcs_real duty[ADCS_MAX_COUPLES]);
uint16_t adcs_crc16(const uint8_t *p, int n);

#endif
