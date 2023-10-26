import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

x = [100, 1000, 10000, 100000, 1000000]
y = [0.22,1.5,14,137,1433]
y1 = [0.30,1.6,14,137,1433]
y2 = [0.61,1.9,15,137,1433]
y3 = [0.35, 1.6, 14, 137, 1433]

y = [ele / 2 for ele in y]
y1 = [ele / 2 for ele in y1]
y2 = [ele / 2 for ele in y2]
y3 = [ele / 2 for ele in y3]

fig, ax = plt.subplots(figsize=(5, 2.7), layout='constrained')
ax.plot(x, y,  '-',  label='Max1')
ax.plot(x, y1, '-.', label='Max2 ($\omega=3$)')
ax.plot(x, y2, '--', label='KRE1')
ax.plot(x, y3, '*', label='KRE2 ($\omega=3$)')
ax.set_xlabel('Input scale ($m$)')
ax.set_ylabel('Communication volume [MB]')
ax.xaxis.set_major_locator(MaxNLocator(integer=True))
ax.legend()

plt.show()
