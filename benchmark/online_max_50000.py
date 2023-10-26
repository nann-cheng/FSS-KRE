import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

x = [20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30]
y1 = [2562, 2682, 2822, 2926, 3067, 3194, 3308, 3447, 3570, 3708, 3812]
y2 = [1572, 1669, 1735, 1800, 1901, 1965, 2030, 2130, 2194, 2257, 2360]
y3 = [1500, 1507, 1575, 1638, 1790, 1796, 1862, 1926, 2078, 2088, 2149]
y4 = [1727, 1729, 1792, 1789, 1855, 2130, 2135, 2199, 2204, 2279, 2538]

y1 = [e/1000 for e in y1]
y2 = [e/1000 for e in y2]
y3 = [e/1000 for e in y3]
y4 = [e/1000 for e in y4]



fig, ax = plt.subplots(figsize=(5, 2.7), layout='constrained')
ax.plot(x, y1, '-',label='Max1')
ax.plot(x, y2, '-.', label='Max2 $(\omega = 3)$')
ax.plot(x, y3, '--', label='Max2 $(\omega = 4)$')
ax.plot(x, y4, ':', label='Max2 $(\omega = 5)$')
ax.set_xlabel('Length of input bits')
ax.set_ylabel('Estimated execution time [s]')
ax.xaxis.set_major_locator(MaxNLocator(integer=True))
ax.legend()

plt.show()
