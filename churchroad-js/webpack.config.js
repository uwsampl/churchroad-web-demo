const path = require('path');
const { library } = require('webpack');

module.exports = {
  entry: './src/index.js',
  output: {
    filename: 'churchroad.js',
    path: path.resolve(__dirname, '../web-demo/static'),
    library: {
      name: 'churchroad',
      type: 'var',
    }
  },
  resolve: {
    extensions: ['.js'],
  },
  externals: {
    'fs/promises': 'fs/promises',
  },
  mode: 'development',
};
