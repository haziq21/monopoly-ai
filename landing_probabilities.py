import matplotlib.pyplot as plt
import numpy as np


def roll(starting_location=0, starting_roll_probability=1, doubles_rolled=0):
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
                    new_board, new_jail = roll(new_location, roll_probability, doubles_rolled + 1)
                    board += new_board
                    jail += new_jail
            
            # Normal turn
            else:
                board[new_location] += roll_probability

    return board, jail


board, jail = roll()
board *= 100
jail *= 100

square_names = ["Go", "Old Kent Road", "Event 1", "Whitechapel Road", "Event 2", "The Angel, Islington", 
      "Euston Road", "Location 1", "Pentonville Road", "Jail", "Pall Mall", "Event 3", 
      "Whitehall", "Northumb'nd Avenue", "Bow Street", "Marlborough Street", "Location 2", "Vine Street", 
      "Free Parking", "Strand", "Event 4", "Fleet Street", "Trafalgar Square", "Leicester Square", 
      "Coventry Street", "Location 3", "Piccadilly", "Go To Jail", "Regent Street", "Event 5", 
      "Oxford Street", "Bond Street", "Event 6", "Park Lane", "Location 4", "Mayfair"]

fig = plt.figure(figsize=(10, 6))
plt.bar(square_names, board)
plt.bar(square_names, jail, bottom=board)
plt.xticks(rotation=60, ha='right')

plt.title('Landing probability from Go')
plt.xlabel('Square')
plt.ylabel('Probability (%)')
plt.legend(['Not going to jail', 'Going to jail'])
fig.tight_layout()

plt.savefig('saved graphs/landing probabilities.png', dpi=200)