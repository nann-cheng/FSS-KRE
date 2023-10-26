import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

x = [20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30]
y1 = [3770, 3964, 4129, 4318, 4503, 4694, 4880, 5065, 5240, 5420, 5614]
y2 = [1573, 1675, 1737, 1804, 1904, 1969, 2033, 2135, 2198, 2266, 2365]
y3 = [1497, 1505, 1560, 1622, 1778, 1781, 1844, 1913, 2061, 2068, 2134]
y4 = [1715, 1716, 1785, 1773, 1849, 2113, 2132, 2173, 2180, 2249, 2506]

y1 = [e/1000 for e in y1]
y2 = [e/1000 for e in y2]
y3 = [e/1000 for e in y3]
y4 = [e/1000 for e in y4]

fig, ax = plt.subplots(figsize=(5, 2.7), layout='constrained')
ax.plot(x, y1, '-', label='KRE1')
ax.plot(x, y2, '-.', label='KRE2 $(\omega = 3)$')
ax.plot(x, y3, '--', label='KRE2 $(\omega = 4)$')
ax.plot(x, y4, ':', label='KRE2 $(\omega = 5)$')
ax.set_xlabel('Length of input bits')
ax.set_ylabel('Estimated execution time [s]')
ax.xaxis.set_major_locator(MaxNLocator(integer=True))
ax.legend()

plt.show()
