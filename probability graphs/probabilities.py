import matplotlib.pyplot as plt
import numpy as np
from celluloid import Camera


def landing_probabilities(starting_location=0, starting_roll_probability=1, doubles_rolled=0):
    board = np.zeros(36)
    jail = np.zeros(36)

    # Roll first die
    for die1 in range(1, 7):
        # Roll second die
        for die2 in range(1, 7):
            # Probability of rolling this exact sequence
            roll_probability = 1/36 * starting_roll_probability
            # Location of player after rolling the dice
            new_location = (starting_location + die1 + die2) % 36

            # 'Go to jail' square
            if new_location == 27:
                jail[new_location] += roll_probability

            # Doubles rolled
            elif die1 == die2:
                # Go to jail if 3 doubles rolled
                if doubles_rolled == 2:
                    jail[new_location] += roll_probability
                # Roll again
                else:
                    new_board, new_jail = landing_probabilities(new_location, roll_probability, doubles_rolled + 1)
                    board += new_board
                    jail += new_jail
            
            # Normal turn
            else:
                board[new_location] += roll_probability

    return board, jail

def double_roll_probabilities():
    board = np.zeros(36)

    # Roll first die
    for die1 in range(1, 7):
        # Roll second die
        for die2 in range(1, 7):
            pass

def movement_probabilities(starting_location=0, starting_roll_probability=1, doubles_rolled=0):
    board = np.zeros(36)
    board[0] = 1
    jail = np.zeros(3)

    while True:
        # Roll first die
        for die1 in range(1, 7):
            # Roll second die
            for die2 in range(1, 7):
                # Probability of rolling this exact sequence
                roll_probability = 1/36 * starting_roll_probability
                # Location of player after rolling the dice
                new_location = (starting_location + die1 + die2) % 36

                # Release players from jail after 3 turns
                jail_release_location = (9 + die1 + die2) % 36
                jail[-1] * roll_probability

                # 'Go to jail' square
                if new_location == 27:
                    jail[0] += roll_probability

                # Doubles rolled
                elif die1 == die2:
                    # Go to jail if 3 doubles rolled
                    if doubles_rolled == 2:
                        jail[0] += roll_probability
                    # Roll again
                    else:
                        new_board, new_jail = landing_probabilities(new_location, roll_probability, doubles_rolled + 1)
                        board += new_board
                        jail += new_jail
                
                # Normal turn
                else:
                    board[new_location] += roll_probability

        yield board, jail

def jail_probability(starting_location=0, starting_roll_probability=1, doubles_rolled=0):
    jail = 0

    # Roll first die
    for die1 in range(1, 7):
        # Roll second die
        for die2 in range(1, 7):
            # Probability of rolling this exact sequence
            roll_probability = 1/36 * starting_roll_probability
            # Location of player after rolling the dice
            new_location = (starting_location + die1 + die2) % 36

            # 'Go to jail' square
            if new_location == 27:
                jail += roll_probability

            # Doubles rolled
            elif die1 == die2:
                # Go to jail if 3 doubles rolled
                if doubles_rolled == 2:
                    jail += roll_probability
                # Roll again
                else:
                    jail += jail_probability(new_location, roll_probability, doubles_rolled + 1)

    return jail


square_names = ["Go", "Old Kent Road", "Event 1", "Whitechapel Road", "Event 2", "The Angel, Islington", 
      "Euston Road", "Location 1", "Pentonville Road", "Jail", "Pall Mall", "Event 3", 
      "Whitehall", "Northumb'nd Avenue", "Bow Street", "Marlborough Street", "Location 2", "Vine Street", 
      "Free Parking", "Strand", "Event 4", "Fleet Street", "Trafalgar Square", "Leicester Square", 
      "Coventry Street", "Location 3", "Piccadilly", "Go To Jail", "Regent Street", "Event 5", 
      "Oxford Street", "Bond Street", "Event 6", "Park Lane", "Location 4", "Mayfair"]


### Landing probabilities from Go ###

board, jail = landing_probabilities()
board *= 100
jail *= 100

fig = plt.figure(figsize=(10, 6))
plt.title('Landing probability from Go')
plt.xlabel('Square')
plt.xticks(rotation=60, ha='right')
plt.ylabel('Probability (%)')

plt.bar(square_names, board)
plt.bar(square_names, jail, bottom=board)
plt.legend(['Not going to jail', 'Going to jail'])
fig.tight_layout()

plt.savefig('probability graphs/landing probabilities.png', dpi=200)


### Jail probabilities from any given square ###

jail_probabilities = np.array([jail_probability(starting_location=i) for i in range(36)])
jail_probabilities *= 100

fig.clf()
plt.title('Probability of going straight to jail from a given square')
plt.xlabel('Square')
plt.ylabel('Probability (%)')
plt.xticks(rotation=60, ha='right')

plt.bar(square_names, jail_probabilities)
plt.savefig('probability graphs/jail probabilities.png', dpi=200)


### Time spent on each square in 20 rolls / moves ###
# This assumes that players try to get out of jail by rolling doubles

fig.clf()
plt.title(f'Landing probabilities after 12 rolls')
plt.xlabel('Square')
plt.ylabel('Probability (%)')
plt.xticks(rotation=60, ha='right')

plt.bar(square_names, board, color='blue')
plt.bar(square_names, jail, bottom=board, color='orange')
plt.legend(['Not going to jail', 'Going to jail'])


# for i in range(11):
#     new_board, new_jail = chain_landing_probabilities(board)
#     board = new_board
#     jail += new_jail
#     print()

# camera = Camera(fig)
# camera.snap()

#     # Changes to anything outside the figure do not get animated
#     # plt.title(f'Landing probabilities after {i+2} rolls')
#     plt.bar(square_names, board, color='blue')
#     plt.bar(square_names, jail, bottom=board, color='orange')
    
#     camera.snap()

# # repeat_delay=2000 doesn't seem to work here...
# animation = camera.animate(interval=1200, repeat_delay=2000)
# animation.save('saved graphs/landing probabilities.gif')
