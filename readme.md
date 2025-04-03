# Pong

In archlinux you might need `sdl2_ttf` package.

# Features to Implement

1. Scoring System
   Track points when the ball passes the paddle (game over) or when it hits bricks
   Display score on screen using SDL2's TTF extension

2. Multiple Balls
   Spawn additional balls after certain score thresholds
   Different ball sizes/speeds for variety

3. Bricks/Breakout Elements
   Add rows of colored bricks at the top that disappear when hit
   Different point values for different brick colors
   Power-ups that drop from broken bricks

4. Power-ups
   Extra wide paddle
   Sticky paddle (ball sticks until you press space)
   Multi-ball
   Laser paddle (shoot to destroy bricks)
   Slow-motion ball

5. Game States
   Main menu screen
   Pause screen
   Game over screen with restart option
   Level progression

6. Visual Enhancements
   Particle effects when ball hits paddle/bricks
   Trail effects behind the ball
   Sound effects using SDL_mixer
   Background music

7. Gameplay Mechanics
   Ball speed increases over time or after paddle hits
   Angled bounces based on where ball hits paddle
   Gravity zones or other environmental effects
   Teleporters or portals

8. Multiplayer
   Two-player mode (one controls top paddle, one controls bottom)
   Network multiplayer support

9. Level Editor
   Create and save custom brick layouts
   Load different levels from files

10. Mobile/Touch Support
    Touch controls for mobile devices
    Accelerometer control

The project is licensed under the [MIT](LICENSE) LICENSE.
