offlineKeyGenBitMaxSize = [
    1556526,
    14840526,
    147680526,
    1476080526,
    2952080526,
    7380080526,
]
offlineKeyGenFSSMaxSize = [
    1538508,
    15398508,
    153998508,
    1539998508,
    3079998508,
    7699998508,
]

print("offlineKeyGenBitMaxSize is: (MB)")
for v in offlineKeyGenBitMaxSize:
    print(v / (1024*1024*2))
print("----")

print("offlineKeyGenFSSMaxSize is: (MB)")
for v in offlineKeyGenFSSMaxSize:
    print(v / (1024 * 1024 * 2))
print("----")


commuBitMax = [5183, 40058, 388808, 3876308, 7751308, 19376308]
commuCMax = [11988, 119988, 1199988, 11999988, 23999988, 59999988]


commuBitKre = [40028, 40178, 43740, 77490, 414990, 3789990]
commuBatchKre = [6688, 6838, 10400, 44150, 381650, 3756650]
commuCKre = [740, 15700]

offlineKeyGenCKre = [82360, 1951480]
offlineKeyGenBitKre = [
    333536,
    548352,
    1749236,
    14619236,
    143319236,
    1430319236,
]
offlineKeyGenBatchKre = [174696, 231896, 1590396, 14460396, 143160396, 1430160396]

print("online BitMax is: (MB\n")
for v in commuBitMax:
    print(v/1024**2)
print("----")

# for v in commuBitMax20:
#     print(v / 1024**2)
# print("----")
print("online fssMax is: (MB\n")

for v in commuCMax:
    print(v / 1024**2)
print("----")


print("online BitKre is: (KB\n")

for v in commuBitKre:
    print(v / 1024**2)
print("----")

print("online BatchKre is: (KB\n")

for v in commuBatchKre:
    print(v / 1024**2)
print("----")

print("online NaiveKre is: (KB\n")
for v in commuCKre:
    print(v / 1024**2)
print("----")

print("offlineKeyGenCKreSize is: (MB)")
for v in offlineKeyGenCKre:
    print(v / (2*1024**2))
print("----")

print("offlineKeyGenBitKre is: (MB)")
for v in offlineKeyGenBitKre:
    print(v / (2*1024**2))
print("----")


print("offlineKeyGenBatchKre is: (MB)")
for v in offlineKeyGenBatchKre:
    print(v / (2 * 1024**2))
print("----")
