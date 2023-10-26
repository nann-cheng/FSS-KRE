import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

x = [1, 2, 3, 4, 5]
y = [(497*10000*e+20*5)/(1000000)   for e in x]
y1 = [(617*10000*e+25*5)/(1000000) for e in x]
y2 = [(737*10000*e+30*5)/(1000000) for e in x]

fig, ax = plt.subplots(figsize=(5, 2.7), layout='constrained')
ax.plot(x, y, '-',label='n = 20')
ax.plot(x, y1, '-.', label='n = 25')
ax.plot(x, y2, '--', label='n = 30')
ax.set_xlabel('Input scale ($ \\times 10^4$)')
ax.set_ylabel('Communication volume [MB]')
ax.xaxis.set_major_locator(MaxNLocator(integer=True))
ax.legend()

plt.show()
