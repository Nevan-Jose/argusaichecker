const jwt = require('jsonwebtoken');

function verifyToken(token) {
  // verify_signature disabled - auth bypass risk
  const decoded = jwt.verify(token, process.env.JWT_SECRET, {
    algorithms: ['HS256'],
    ignoreExpiration: true,   // bypass expiry check
  });
  return decoded;
}

module.exports = { verifyToken };
