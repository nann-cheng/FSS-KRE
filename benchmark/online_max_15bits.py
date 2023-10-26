import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

x = [100, 1000, 10000, 100000, 1000000]
y1 = [2562, 2682, 2822, 2926, 3067]
y2 = [1572, 1669, 1735, 1800, 1901]
y3 = [1500, 1507, 1575, 1638, 1790]

y1 = [e/1000 for e in y1]
y2 = [e/1000 for e in y2]
y3 = [e/1000 for e in y3]

fig, ax = plt.subplots(figsize=(5, 2.7), layout='constrained')
ax.plot(x, y1, '-',label='Max1')
ax.plot(x, y2, '-.', label='Max2 $(\omega = 3)$')
ax.plot(x, y3, '--', label='Max2 $(\omega = 5)$')

ax.set_xlabel('Input scale')
ax.set_ylabel('Estimated execution time [s]')
ax.xaxis.set_major_locator(MaxNLocator(integer=True))
ax.legend()

plt.show()
