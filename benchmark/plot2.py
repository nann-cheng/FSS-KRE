import matplotlib.pyplot as plt
import numpy as np

def max1commuFunc(n):
	return 3*(n**2) -n

# Define the range of x values
x = np.logspace(np.log10(100),np.log10(10000000),num=10)

shape = (10,)

# Define the linear equations

# Note: COU18 online rounds counts the addtion of each secure comparison cost, 
# along with B2A round, one more round for two multiplication
y1 = x*(622+1+2*32)/1024**2

# Note: Damgaard19 online rounds counts the addtion of each secure comparison cost, 
# along with one more round for two multiplication
y2 = x*(6*32-8+2*32)/1024**2
y3 = x*(3*32)/1024**2
y4 = (x*32+max1commuFunc(32))/1024**2

y33 = x*(3*(32+40))/1024**2



# Plotting the lines with different line styles and markers
plt.plot(x, y33, linestyle=':', marker='o', label='Catrina10, n=32')
plt.plot(x, y1, linestyle='-', marker='o', label='COU18, n=32')
plt.plot(x, y2, linestyle='--', marker='o', label='Damgård19, n=32')
plt.plot(x, y3, linestyle=':', marker='o', label='Boyle19, n=32')
plt.plot(x, y4, linestyle='-.', marker='o', label='Our $\Pi_{Max}$, n=32')


# plt.xscale('log')
plt.yscale('log')

plt.xscale('log',base=2)


# Adding a legend to differentiate the lines
plt.legend(loc='upper left')

# Adding titles and labels
# plt.title('Concrete rounds')
plt.xlabel('Input scale m')
plt.ylabel('Online commu. [MB]')

# Show the plot
plt.show()