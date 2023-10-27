import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

x = [100, 1000, 10000, 100000, 1000000]
y= [1, 5.99, 54, 500, 5110]
y1 = [1.68, 6.3, 50, 518, 5106]
y2 = [2.05, 6.77, 50, 513, 5246]
y3 = [2.6, 6.38, 51, 500, 5144]

fig, ax = plt.subplots(figsize=(5, 2.7), layout='constrained')
ax.plot(x, y,  '-',  label='Max1')
ax.plot(x, y1, '-.', label='Max2 ($\omega=3$)')
ax.plot(x, y2, '--', label='KRE1')
ax.plot(x, y3, '*', label='KRE2 ($\omega=3$)')
ax.set_xlabel('Input scale ($m$)')
ax.set_ylabel('Execution time [ms]')
ax.xaxis.set_major_locator(MaxNLocator(integer=True))
ax.legend()
plt.xscale("log")  # Optional, to set the x-axis to a logarithmic scale
plt.show()
