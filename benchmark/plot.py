import matplotlib.pyplot as plt
import numpy as np


# Define the range of x values
x = np.linspace(100, 10000000,num=10)
shape = (10,)

# Define the linear equations

# Note: COU18 online rounds counts the addtion of each secure comparison cost, 
# along with B2A round, one more round for two multiplication
y1 = (9+2)*np.ceil(np.log2(x))

# Note: Damgaard19 online rounds counts the addtion of each secure comparison cost, 
# along with one more round for two multiplication
y2 = (7+1)*np.ceil(np.log2(x))
y3 = (1+1)*np.ceil(np.log2(x))
y4 = np.full(shape,2*32-1)


y11 = (10+2)*np.ceil(np.log2(x))
y22 = (10+1)*np.ceil(np.log2(x))
y44 = np.full(shape,2*64-1)


# Plotting the lines with different line styles and markers
plt.plot(x, y1, linestyle='-', marker='x', label='COU18, n=32')
plt.plot(x, y2, linestyle='--', marker='x', label='Damgaard19, n=32')
plt.plot(x, y3, linestyle=':', marker='x', label='Boyle19, n=32/64')
plt.plot(x, y4, linestyle='-.', marker='x', label='$\Pi_{Max1}$, n=32')

plt.plot(x, y11, linestyle='-', marker='^', label='COU18, n=64')
plt.plot(x, y22, linestyle='--', marker='^', label='Damgaard19, n=64')
plt.plot(x, y44, linestyle='-.', marker='^', label='$\Pi_{Max1}$, n=64')

plt.xscale('log')


# Adding a legend to differentiate the lines
plt.legend(loc='upper left')

# Adding titles and labels
# plt.title('Concrete rounds')
plt.xlabel('Input scale m')
plt.ylabel('Online rounds')

# Show the plot
plt.show()
