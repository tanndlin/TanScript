pipeline {
    agent any

    environment {
        GITHUB_TOKEN = credentials('GITHUB_TOKEN')
        CARGO = 'docker run --rm -v jenkins_jenkins_home:/var/jenkins_home -v cargo-registry-cache:/usr/local/cargo/registry -w /var/jenkins_home/workspace/TanScript rust:latest sh -c'
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
                sh '''
                curl -L \
                -X POST \
                -H "Accept: application/vnd.github+json" \
                -H "Authorization: Bearer $GITHUB_TOKEN" \
                -H "X-GitHub-Api-Version: 2022-11-28" \
                https://api.github.com/repos/tanndlin/TanScript/statuses/$GIT_COMMIT \
                -d '{"state":"pending","description":"Build in progress","context":"Jenkins"}'
                '''
            }
        }

        stage('Build') {
            steps {
                sh '''
                $CARGO 'cargo build --release'
                '''
            }
        }

        stage('Lint') {
            steps {
                catchError(buildResult: 'FAILURE', stageResult: 'FAILURE') {
                    sh '''
                    $CARGO 'rustup component add rustfmt clippy && cargo fmt -- --check && cargo clippy -- -D clippy::pedantic'
                    '''
                }
            }
        }

        stage('Test') {
            steps {
                sh '''
                $CARGO 'cargo test'
                '''
            }
        }
    }

    post {
        success {
            sh '''
            curl -L \
            -X POST \
            -H "Accept: application/vnd.github+json" \
            -H "Authorization: Bearer $GITHUB_TOKEN" \
            -H "X-GitHub-Api-Version: 2022-11-28" \
            https://api.github.com/repos/tanndlin/TanScript/statuses/$GIT_COMMIT \
            -d '{"state":"success","description":"Build succeeded","context":"Jenkins"}'
            '''
        }
        failure {
            sh '''
            curl -L \
            -X POST \
            -H "Accept: application/vnd.github+json" \
            -H "Authorization: Bearer $GITHUB_TOKEN" \
            -H "X-GitHub-Api-Version: 2022-11-28" \
            https://api.github.com/repos/tanndlin/TanScript/statuses/$GIT_COMMIT \
            -d '{"state":"failure","description":"Build failed","context":"Jenkins"}'
            '''
        }
    }
}
