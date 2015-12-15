## Tak-server

This is me playing with Iron, and an attempt to use tak-rs. This code is
running at tak-server.herokuapp.com.

## Usage

You can view all of the games being played at the homepage.

You can create, join, or spectate a game by going to
tak-server.herokuapp.com/game/gameName. There is a fallback text interface at
tak-server.herokuapp.com/game/gameName/text.

Once in a game, to place a stone, you click on the type of stone you want to
play (displayed on the sides of the board), and then the location you want to
place the stone.

To slide a pile, click on the pile you want to slide, and then, from the bottom
of the stack up, click on the location you want each stone to end up.

Getting these clicks right is currently pretty hard, since there is no display
for the current input state.

## TODO

1. Game page: Display when game is won.
2. Game page: Highlight things to make it more obvious what to click next.
3. Game page: Show in-progress slides?
