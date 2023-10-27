import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

x = [100, 1000, 10000, 100000, 1000000]
y1 = [4.23, 4.26, 4.34, 5.11, 10.92]#commu: [487,2174,19049,187799,1875299]
y2 = [1.98, 2.01, 2.24, 3.76, 23.69]#commu: [458,2145,19020,187770,1875270]

y1 = [e/1000 for e in y1]
y2 = [e/1000 for e in y2]

fig, ax = plt.subplots(figsize=(5, 2.7), layout='constrained')
ax.plot(x, y1, '-',label='Max1')
ax.plot(x, y2, '-.', label='Max2 $(\omega = 3)$')

ax.set_xlabel('Input scale')
ax.set_ylabel('Estimated execution time [s]')
ax.xaxis.set_major_locator(MaxNLocator(integer=True))
ax.legend()

plt.show()
