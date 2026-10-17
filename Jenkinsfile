def githubStatus(String state, String description, String context = 'Jenkins') {
    sh """
    curl -sL \\
    -X POST \\
    -H "Accept: application/vnd.github+json" \\
    -H "Authorization: Bearer \$GITHUB_TOKEN" \\
    -H "X-GitHub-Api-Version: 2022-11-28" \\
    https://api.github.com/repos/tanndlin/TanScript/statuses/\$GIT_COMMIT \\
    -d '{"state":"${state}","description":"${description}","context":"${context}"}'
    """
}

pipeline {
    agent any

    options {
        timeout(time: 30, unit: 'MINUTES')
        timestamps()
    }

    triggers {
        cron('H 3 * * *')
    }

    environment {
        GITHUB_TOKEN = credentials('GITHUB_TOKEN')
        DOCKER_VOLS = '-v jenkins_jenkins_home:/var/jenkins_home -v cargo-registry-cache:/usr/local/cargo/registry'
        RUST_IMAGE = 'rust:1.97'
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
                githubStatus('pending', 'Build in progress')
            }
        }

        stage('Build') {
            steps {
                sh '''
                docker run --rm $DOCKER_VOLS -w $WORKSPACE $RUST_IMAGE \
                    sh -c "cargo build --all-targets"
                '''
            }
        }

        stage('Checks') {
            parallel {
                stage('Format') {
                    steps {
                        catchError(buildResult: 'FAILURE', stageResult: 'FAILURE') {
                            sh '''
                            docker run --rm $DOCKER_VOLS -w $WORKSPACE $RUST_IMAGE \
                                sh -c "rustup component add rustfmt && cargo fmt -- --check"
                            '''
                        }
                    }
                }

                stage('Lint') {
                    steps {
                        catchError(buildResult: 'FAILURE', stageResult: 'FAILURE') {
                            sh '''
                            docker run --rm $DOCKER_VOLS -w $WORKSPACE $RUST_IMAGE \
                                sh -c "rustup component add clippy && cargo clippy --all-targets -- -D clippy::pedantic"
                            '''
                        }
                    }
                }

                stage('Test') {
                    steps {
                        sh '''
                        docker run --rm $DOCKER_VOLS -w $WORKSPACE -e KEEP_GOLDEN=1 $RUST_IMAGE \
                            sh -c "curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C /usr/local/cargo/bin \
                                && rm -rf target/tmp/golden \
                                && cargo nextest run --profile ci"
                        '''
                    }
                    post {
                        always {
                            junit allowEmptyResults: true, testResults: 'target/nextest/ci/junit.xml'
                            archiveArtifacts artifacts: 'target/tmp/golden/**/*.asm', allowEmptyArchive: true
                        }
                    }
                }

                stage('Coverage') {
                    steps {
                        sh '''
                        docker run --rm $DOCKER_VOLS -w $WORKSPACE $RUST_IMAGE \
                            sh -c "curl -LsSf https://github.com/xd009642/tarpaulin/releases/latest/download/cargo-tarpaulin-x86_64-unknown-linux-gnu.tar.gz | tar zxf - -C /usr/local/cargo/bin \
                                && cargo tarpaulin --engine llvm --skip-clean --target-dir target/tarpaulin --out Xml --output-dir target/tarpaulin"
                        '''
                    }
                    post {
                        success {
                            recordCoverage(
                                tools: [[parser: 'COBERTURA', pattern: 'target/tarpaulin/cobertura.xml']],
                                sourceCodeRetention: 'EVERY_BUILD'
                            )
                        }
                    }
                }
            }
        }

        stage('Release') {
            when {
                allOf {
                    expression { env.GIT_BRANCH == 'master' || env.GIT_BRANCH == 'origin/master' }
                    expression { currentBuild.currentResult == 'SUCCESS' }
                }
            }
            steps {
                sh '''
                docker run --rm $DOCKER_VOLS -w $WORKSPACE $RUST_IMAGE \
                    sh -c "cargo build --release"
                '''
                archiveArtifacts artifacts: 'target/release/tanscript', fingerprint: true
            }
        }

        stage('Fuzz') {
            when {
                triggeredBy 'TimerTrigger'
            }
            steps {
                sh '''
                docker run --rm $DOCKER_VOLS -w $WORKSPACE $RUST_IMAGE \
                    sh -c "rustup toolchain install nightly --profile minimal \
                        && cargo install cargo-fuzz --locked \
                        && mkdir -p fuzz/corpus/compile \
                        && cargo +nightly fuzz run compile fuzz/corpus/compile tests/golden -- -max_total_time=600 -timeout=10"
                '''
            }
            post {
                always {
                    archiveArtifacts artifacts: 'fuzz/artifacts/**', allowEmptyArchive: true
                }
            }
        }
    }

    post {
        success {
            githubStatus('success', 'Build succeeded')
        }
        unsuccessful {
            githubStatus('failure', 'Build failed')
        }
    }
}
