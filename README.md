## Dependencies

- node.js => v20

## Installation

1. Download the according executable file from the [releases](https://github.com/SimaoMoreira5228/manga-vault/releases/latest) page.

   - manga-vault.exe => Windows
   - manga-vault => Linux

2. Run the executable file.

   - After running the executable file, some files and folders will be created in the same directory as the executable file.

3. The default routes are:

   - [website](http://localhost:5227) => http://localhost:5227
   - [api](http://localhost:5228) => http://localhost:5228
   - [websocket](http://localhost:5229) => http://localhost:5229

4. You can change the routes (and other settings) by editing the `config.json` file.

5. Some website configurations can be changed by editing the `.env` file in the `website` folder.
   - if the `.env` file does not exist, create it.
   - Example:
     ```env
      API_IP=localhost
      API_PORT=5228
      BODY_SIZE_LIMIT=5000000 # 5MB
     ```
     - This sets the API IP to `localhost`, the API port to `5228`, and the body size limit to `5MB`.
