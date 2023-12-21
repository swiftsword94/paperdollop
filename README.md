# Paperdollop

Make paperdolls for any character with a single 32x32 weapon image.
This takes the effort out of having to manually move each image a couple pixels and does it for you.

This handles:
- rotation
- mirroring
- scaling
- offsets

Instructions:
- open in command line,
- update the settings file 
- Try calculating the scaling factor based on 1-(2 * (abs(maximum offset size) / 32)) for best results.
  The maximum offset size is the maximum number in the "row" property.
  32 is really just the input image size which is assumed to be a 32x32 image.
- provide the paths for:
  - character_file
  - output_directory
  - settings
  - Either one of the two:
    - item_file - for single item paperdolls
    - item_directory - for creating paperdolls for multiple input item inputs in the same directory
  

settings.json format: 

    {
      "matrix": [
        {
          "angle": -0.7853982,
          "mirror_x": false,
          "mirror_y": false,
          "scaling_factor": 1.0,
          "row": [[-6,0], [-6,1], [-6,0], [-6,-2]]
        },
        {
          "angle": 0,
          "mirror_x": false,
          "mirror_y": false,
          "scaling_factor": 0.9375,
          "row": [[0,0], [-2,-1], [0,0], [2,0]]
        },
        {
          "angle": 0,
          "mirror_x": true,
          "mirror_y": false,
          "scaling_factor": 0.9375,
          "row": [[0,0], [0,1], [0,0], [0,-2]]
        },
        {
          "angle": 0,
          "mirror_x": false,
          "mirror_y": false,
          "scaling_factor": 0.9375,
          "row": [[0,0], [0,1], [0,0], [0,-2]]
        }
      ]
    }