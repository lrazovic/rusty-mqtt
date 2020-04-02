/*
 * Copyright (C) 2015 Freie Universität Berlin
 *
 * This file is subject to the terms and conditions of the GNU Lesser
 * General Public License v2.1. See the file LICENSE in the top level
 * directory for more details.
 */

/**
 * @ingroup     examples
 * @{
 *
 * @file
 * @brief       Example application for demonstrating RIOT's MQTT-SN library
 *              emCute
 *
 * @author      Hauke Petersen <hauke.petersen@fu-berlin.de>
 *
 * @}
 */

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include "shell.h"
#include "random.h"
#include "xtimer.h"
#include "msg.h"
#include "net/emcute.h"
#include "net/ipv6/addr.h"
// #include "lpsxxx.h"
// #include "lpsxxx_params.h"
// #include "lpsxxx_internal.h"

#ifndef EMCUTE_ID
#define EMCUTE_ID ("gertrud")
#endif
#define EMCUTE_PORT (1883U)
#define EMCUTE_PRIO (THREAD_PRIORITY_MAIN - 1)

#define NUMOFSUBS (16U)
#define TOPIC_MAXLEN (64U)

static char stack[THREAD_STACKSIZE_DEFAULT];
static msg_t queue[8];

static emcute_sub_t subscriptions[NUMOFSUBS];

static void *emcute_thread(void *arg)
{
    (void)arg;
    emcute_run(EMCUTE_PORT, EMCUTE_ID);
    return NULL; /* should never be reached */
}

static unsigned get_qos(const char *str)
{
    int qos = atoi(str);
    switch (qos)
    {
    case 1:
        return EMCUTE_QOS_1;
    case 2:
        return EMCUTE_QOS_2;
    default:
        return EMCUTE_QOS_0;
    }
}

static int cmd_con(int argc, char **argv)
{
    sock_udp_ep_t gw = {.family = AF_INET6, .port = EMCUTE_PORT};
    char *topic = NULL;
    char *message = NULL;
    size_t len = 0;

    if (argc < 2)
    {
        printf("usage: %s <ipv6 addr> [port] [<will topic> <will message>]\n",
               argv[0]);
        return 1;
    }

    /* parse address */
    if (ipv6_addr_from_str((ipv6_addr_t *)&gw.addr.ipv6, argv[1]) == NULL)
    {
        printf("error parsing IPv6 address\n");
        return 1;
    }

    if (argc >= 3)
    {
        gw.port = atoi(argv[2]);
    }
    if (argc >= 5)
    {
        topic = argv[3];
        message = argv[4];
        len = strlen(message);
    }

    if (emcute_con(&gw, true, topic, message, len, 0) != EMCUTE_OK)
    {
        printf("error: unable to connect to [%s]:%i\n", argv[1], (int)gw.port);
        return 1;
    }
    printf("Successfully connected to gateway at [%s]:%i\n",
           argv[1], (int)gw.port);

    return 0;
}

static int cmd_discon(int argc, char **argv)
{
    (void)argc;
    (void)argv;

    int res = emcute_discon();
    if (res == EMCUTE_NOGW)
    {
        puts("error: not connected to any broker");
        return 1;
    }
    else if (res != EMCUTE_OK)
    {
        puts("error: unable to disconnect");
        return 1;
    }
    puts("Disconnect successful");
    return 0;
}

static int cmd_pub(int argc, char **argv)
{
    emcute_topic_t t;
    unsigned flags = EMCUTE_QOS_0;

    if (argc < 3)
    {
        printf("usage: %s <topic name> <data> [QoS level]\n", argv[0]);
        return 1;
    }

    /* parse QoS level */
    if (argc >= 4)
    {
        flags |= get_qos(argv[3]);
    }

    printf("pub with topic: %s and name %s and flags 0x%02x\n", argv[1], argv[2], (int)flags);

    /* step 1: get topic id */
    t.name = argv[1];
    if (emcute_reg(&t) != EMCUTE_OK)
    {
        puts("error: unable to obtain topic ID");
        return 1;
    }

    /* step 2: publish data */
    if (emcute_pub(&t, argv[2], strlen(argv[2]), flags) != EMCUTE_OK)
    {
        printf("error: unable to publish data to topic '%s [%i]'\n",
               t.name, (int)t.id);
        return 1;
    }

    printf("Published %i bytes to topic '%s [%i]'\n",
           (int)strlen(argv[2]), t.name, t.id);

    return 0;
}

static int cmd_fpub(int argc, char **argv)
{
    emcute_topic_t t;
    unsigned flags = EMCUTE_QOS_0;

    if (argc < 3)
    {
        printf("usage: %s <topic name> <device id> [QoS level]\n", argv[0]);
        return 1;
    }

    /* parse QoS level */
    if (argc >= 4)
    {
        flags |= get_qos(argv[3]);
    }

    /* step 1: get topic id */
    t.name = argv[1];
    if (emcute_reg(&t) != EMCUTE_OK)
    {
        puts("error: unable to obtain topic ID");
        return 1;
    }

    while (1)
    {
        int device_id = atoi(argv[2]);
        int temperature = random_uint32_range(0, 100);
        int h = random_uint32_range(0, 100);
        int wd = random_uint32_range(0, 360);
        int wi = random_uint32_range(0, 100);
        int rh = random_uint32_range(0, 50);
        char values[6] = {device_id, temperature, h, wd, wi, rh};
        /* step 2: publish data */
        if (emcute_pub(&t, values, sizeof(values), flags) != EMCUTE_OK)
        {
            printf("error: unable to publish data to topic '%s [%i]'\n",
                   t.name, (int)t.id);
            return 1;
        }

        printf("Published %i bytes to topic '%s [%i]'\n",
               sizeof(values), t.name, t.id);
        xtimer_sleep(5);
    }

    return 0;
}

/*
static int cmd_temp_pub(int argc, char **argv)
{
    emcute_topic_t t;
    lpsxxx_t dev;
    unsigned flags = EMCUTE_QOS_0;

    if (argc < 3)
    {
        printf("usage: %s <topic name> <device id> [QoS level]\n", argv[0]);
        return 1;
    }

    // parse QoS level 
    if (argc >= 4)
    {
        flags |= get_qos(argv[3]);
    }

    // step 1: get topic id
    t.name = argv[1];
    if (emcute_reg(&t) != EMCUTE_OK)
    {
        puts("error: unable to obtain topic ID");
        return 1;
    }
    if (lpsxxx_init(&dev, &lpsxxx_params[0]) != LPSXXX_OK)
    {
        puts("Initialization failed");
        return 1;
    }

    int16_t temp;
    while (1)
    {
        lpsxxx_enable(&dev);
        xtimer_sleep(1); // wait a bit for the measurements to complete
        lpsxxx_read_temp(&dev, &temp);
        lpsxxx_disable(&dev);
        int temp_abs = temp / 100;
        int device_id = atoi(argv[2]);
        int temperature = temp_abs;
        int h = random_uint32_range(0, 100);
        int wd = random_uint32_range(0, 360);
        int wi = random_uint32_range(0, 100);
        int rh = random_uint32_range(0, 50);
        char values[6] = {device_id, temperature, h, wd, wi, rh};
        // step 2: publish data
        if (emcute_pub(&t, values, sizeof(values), flags) != EMCUTE_OK)
        {
            printf("error: unable to publish data to topic '%s [%i]'\n",
                   t.name, (int)t.id);
            return 1;
        }

        printf("Published %i bytes to topic '%s [%i]'\n",
               sizeof(values), t.name, t.id);
        xtimer_sleep(5);
    }

    return 0;
}

static int cmd_read(int argc, char **argv)
{
    lpsxxx_t dev;
    (void)argc;
    (void)argv;

    printf("Test application for %s pressure sensor\n\n", LPSXXX_SAUL_NAME);
    printf("Initializing %s sensor\n", LPSXXX_SAUL_NAME);

    if (lpsxxx_init(&dev, &lpsxxx_params[0]) != LPSXXX_OK)
    {
        puts("Initialization failed");
        return 1;
    }

    uint16_t pres;
    int16_t temp;
    while (1)
    {
        lpsxxx_enable(&dev);
        xtimer_sleep(1);

        lpsxxx_read_temp(&dev, &temp);
        lpsxxx_read_pres(&dev, &pres);
        lpsxxx_disable(&dev);

        int temp_abs = temp / 100;
        temp -= temp_abs * 100;

        printf("Pressure value: %ihPa - Temperature: %2i.%02i°C\n",
               pres, temp_abs, temp);
    }

    return 0;
}
*/

static const shell_command_t shell_commands[] = {
    {"con", "connect to MQTT broker", cmd_con},
    {"discon", "disconnect from the current broker", cmd_discon},
    {"pub", "publish something", cmd_pub},
    {"fpub", "publish random data", cmd_fpub},
    //{"read", "read device sensors", cmd_read},
    //{"tpub", "publish random data + real temperature", cmd_temp_pub},
    {NULL, NULL, NULL}};

int main(void)
{
    puts("MQTT-SN example application\n");
    puts("Type 'help' to get started. Have a look at the README.md for more"
         "information.");

    /* the main thread needs a msg queue to be able to run `ping6`*/
    msg_init_queue(queue, ARRAY_SIZE(queue));

    /* initialize our subscription buffers */
    memset(subscriptions, 0, (NUMOFSUBS * sizeof(emcute_sub_t)));

    /* start the emcute thread */
    thread_create(stack, sizeof(stack), EMCUTE_PRIO, 0,
                  emcute_thread, NULL, "emcute");

    /* start shell */
    char line_buf[SHELL_DEFAULT_BUFSIZE];
    shell_run(shell_commands, line_buf, SHELL_DEFAULT_BUFSIZE);

    /* should be never reached */
    return 0;
}
