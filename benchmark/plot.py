import matplotlib.pyplot as plt
import numpy as np


# Define the range of x values
x = np.logspace(np.log10(100),np.log10(10000000),num=10)

shape = (10,)

# Define the linear equations

# Note: COU18 online rounds counts the addtion of each secure comparison cost, 
# along with B2A round, one more round for two multiplication
y1 = (9+2)*np.ceil(np.log2(x))

# Note: Damgaard19 online rounds counts the addtion of each secure comparison cost, 
# along with one more round for two multiplication
y2 = (7+1)*np.ceil(np.log2(x))
y3 = (1+1)*np.ceil(np.log2(x))
# y00000 = (3+1)*np.ceil(np.log2(x))
y4 = np.full(shape,32+1)


y11 = (10+2)*np.ceil(np.log2(x))
y22 = (10+1)*np.ceil(np.log2(x))
y44 = np.full(shape,64+1)


# Plotting the lines with different line styles and markers
plt.plot(x, y1, linestyle='-', marker='o', label='COU18, n=32')
plt.plot(x, y2, linestyle='--', marker='o', label='Catrina10/Damgård19, n=32')
plt.plot(x, y3, linestyle=':', marker='o', label='Boyle19, n=32/64')
# plt.plot(x, y00000, linestyle='-', marker='o', label='Catrina10, n=32/64')

plt.plot(x, y4, linestyle='-.', marker='x', label='Our $\Pi_{Max}$, n=32')

plt.plot(x, y11, linestyle='-', marker='^', label='COU18, n=64')
plt.plot(x, y22, linestyle='--', marker='^', label='Catrina10/Damgård19, n=64')
plt.plot(x, y44, linestyle='-.', marker='^', label='Our $\Pi_{Max}$, n=64')

plt.xscale('log',base=2)


# Adding a legend to differentiate the lines
plt.legend(loc='upper left')

# Adding titles and labels
# plt.title('Concrete rounds')
plt.xlabel('Input scale m')
plt.ylabel('Online rounds')

# Show the plot
plt.show()
