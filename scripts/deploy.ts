import { execSync } from 'child_process';

function main() {
    console.log("Starting deployment for Bastion Protocol...");
    
    console.log("Building Anchor programs...");
    execSync('anchor build', { stdio: 'inherit' });
    
    console.log("Deploying to active network...");
    execSync('anchor deploy', { stdio: 'inherit' });
    
    console.log("Deployment complete.");
}

main();
