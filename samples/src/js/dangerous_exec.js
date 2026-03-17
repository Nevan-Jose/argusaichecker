const { exec } = require('child_process');

function runReport(reportName) {
  // Dangerous: reportName from user request is passed directly to shell
  exec(`generate-report --name ${reportName}`, (err, stdout) => {
    if (err) throw err;
    console.log(stdout);
  });
}

module.exports = { runReport };
