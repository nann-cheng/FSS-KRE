import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

x = [20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30]
y1 = [14025, 15297, 15309, 16335, 18477,
      18257, 19714, 18850, 20184, 20610, 21337]
y2 = [23128, 23597, 25473, 25907, 27770,
      28383, 30041, 30587, 32396, 32850, 34779]

y1 = [e/1000 for e in y1]
y2 = [e/1000 for e in y2]


fig, ax = plt.subplots(figsize=(5, 2.7), layout='constrained')
ax.plot(x, y1, '-',label='KRE1')
ax.plot(x, y2, '-.', label='KRE2 $(\omega = 2)$')
ax.set_xlabel('Length of input bits')
ax.set_ylabel('Estimated execution time [s]')
ax.xaxis.set_major_locator(MaxNLocator(integer=True))
ax.legend()

plt.show()
