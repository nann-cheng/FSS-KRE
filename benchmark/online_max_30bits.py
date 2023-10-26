import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

x = [20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30]
y1 = [13210, 13583, 14642, 13961, 14712,
      16200, 18517, 16949, 19618, 19855, 22038]
y2 = [23048, 23464, 25494, 25817, 27684,
      28262, 29988, 30522, 32220, 32796, 34736]

y1 = [e/1000 for e in y1]
y2 = [e/1000 for e in y2]


InCmpRounds = 16
WAN_DELAY = 60
mpSpdzTime = 73.2339+WAN_DELAY*InCmpRounds*20/1000
mpSpdzCommu = 4359.18
print("MP-SPDZ counterpart requires communication time:", mpSpdzTime)
print("MP-SPDZ counterpart requires communication vlume:", mpSpdzCommu, ' MB')

fig, ax = plt.subplots(figsize=(5, 2.7), layout='constrained')
ax.plot(x, y1, '-',label='Max1')
ax.plot(x, y2, '-.', label='Max2 $(\omega = 2)$')
ax.set_xlabel('Length of input bits')
ax.set_ylabel('Estimated execution time [s]')
ax.xaxis.set_major_locator(MaxNLocator(integer=True))
ax.legend()

plt.show()
