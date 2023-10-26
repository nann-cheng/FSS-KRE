import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

x = [100, 1000, 10000, 100000, 1000000]
y= [32, 53, 73, 96, 120]
y1 = [34, 75, 96, 125, 157]
y2 = [40, 89, 109, 145, 180]
y3 = [40, 89, 109, 145, 180]

fig, ax = plt.subplots(figsize=(5, 2.7), layout='constrained')
ax.plot(x, y,  '-',  label='Max1')
ax.plot(x, y1, '-.', label='Max2 ($\omega=3$)')
ax.plot(x, y2, '--', label='KRE1')
ax.plot(x, y3, '*', label='KRE2 ($\omega=3$)')
ax.set_xlabel('Input scale ($m$)')
ax.set_ylabel('Execution time [ms]')
ax.xaxis.set_major_locator(MaxNLocator(integer=True))
ax.legend()

plt.show()
