import matplotlib.pyplot as plt
import numpy as np
from celluloid import Camera


def landing_probabilities(starting_location=0, starting_roll_probability=1, doubles_rolled=0):
    board = np.zeros(36)
    jail = np.zeros(36)

    # TODO: return if starting location is 'Go To Jail' square?

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
                else:
                    new_board, new_jail = landing_probabilities(new_location, roll_probability, doubles_rolled + 1)
                    board += new_board
                    jail += new_jail
            
            # Normal turn
            else:
                board[new_location] += roll_probability

    return board, jail

def jail_probability(starting_location=0, starting_roll_probability=1, doubles_rolled=0):
    jail = 0

    # TODO: return if starting location is 'Go To Jail' square?

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
                else:
                    jail += jail_probability(new_location, roll_probability, doubles_rolled + 1)

    return jail

def chain_landing_probabilities(board_probabilities):
    new_board = np.zeros(36)
    new_jail = np.zeros(36)

    for i in range(36):
        board_from_i, jail_from_i = landing_probabilities(starting_location=i)
        new_board += board_from_i * board_probabilities[i]
        new_jail += jail_from_i * board_probabilities[i]
    
    return new_board, new_jail


square_names = ["Go", "Old Kent Road", "Event 1", "Whitechapel Road", "Event 2", "The Angel, Islington", 
      "Euston Road", "Location 1", "Pentonville Road", "Jail", "Pall Mall", "Event 3", 
      "Whitehall", "Northumb'nd Avenue", "Bow Street", "Marlborough Street", "Location 2", "Vine Street", 
      "Free Parking", "Strand", "Event 4", "Fleet Street", "Trafalgar Square", "Leicester Square", 
      "Coventry Street", "Location 3", "Piccadilly", "Go To Jail", "Regent Street", "Event 5", 
      "Oxford Street", "Bond Street", "Event 6", "Park Lane", "Location 4", "Mayfair"]

board, jail = landing_probabilities()
board *= 100
jail *= 100

fig = plt.figure(figsize=(10, 6))

plt.title(f'Landing probabilities after 12 rolls')
plt.xlabel('Square')
plt.ylabel('Probability (%)')
plt.xticks(rotation=60, ha='right')

plt.bar(square_names, board, color='blue')
plt.bar(square_names, jail, bottom=board, color='orange')
plt.legend(['Not going to jail', 'Going to jail'])
fig.tight_layout()

camera = Camera(fig)
camera.snap()

for i in range(11):
    new_board, new_jail = chain_landing_probabilities(board)
    board = new_board
    jail += new_jail

    # Changes to anything outside the figure do not get animated
    # plt.title(f'Landing probabilities after {i+2} rolls')
    plt.bar(square_names, board, color='blue')
    plt.bar(square_names, jail, bottom=board, color='orange')
    
    camera.snap()

# repeat_delay=2000 doesn't seem to work here...
animation = camera.animate(interval=1200, repeat_delay=2000)
animation.save('saved graphs/landing probabilities.gif')
