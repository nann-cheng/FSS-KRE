InCmpRounds = 16
WAN_DELAY = 60
LAN_DELAY = 1

timeList = [4.61351, 18.2729, 73.2339, 292.923]
commuData = [273.97, 1091.31, 4359.18, 17432.1]

Length = len(timeList)

LAN_STRING="$"
WAN_STRING="$"
for i in range(Length):
    mpSpdzTime = timeList[i]/2+LAN_DELAY*InCmpRounds*16/1000
    LAN_STRING += "{}$&$".format(mpSpdzTime)
    print("LAN MP-SPDZ requires communication time:", mpSpdzTime)

    mpSpdzTime = timeList[i]/2+WAN_DELAY*InCmpRounds*16/1000
    WAN_STRING += "{}$&$".format(mpSpdzTime)
    print("WAN MP-SPDZ requires communication time:", mpSpdzTime)

    print("MP-SPDZ requires communication vlume:", commuData[i], ' MB')

    print("\n\n")

print(LAN_STRING)
print(WAN_STRING)
